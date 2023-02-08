
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

enum JustRateAction
{
    AddClient(u64, simple_websockets::Responder),
    RemoveClient(u64),
    BroadCastMessageToAllClients(PublisherMessage)
}

pub fn run(
    udp_socket: UdpSocket,
    websocket_event_hub: simple_websockets::EventHub,
) -> std::io::Result<()> {
    let mut reusable_buffer: [u8; 10000] = [0; 10000];
    let (tx, rx): (
        Sender<JustRateAction>,
        Receiver<JustRateAction>,
    ) = mpsc::channel();

    //  Websocket events => Channel
    let websocket_tx = tx.clone();
    std::thread::spawn(move || loop {
        loop {
            let event = websocket_event_hub.poll_event();
            match event {
                simple_websockets::Event::Connect(client_id, responder) => {
                    websocket_tx.send(JustRateAction::AddClient(client_id, responder)).expect("Sending Connect to websocket_tx failed");
                    println!("A client connected with id #{}", client_id);
                }
                simple_websockets::Event::Disconnect(client_id) => {
                    websocket_tx.send(JustRateAction::RemoveClient(client_id)).expect("Sending Disconnect to websocket_tx failed");
                    println!("Client #{} disconnected.", client_id);
                }
                simple_websockets::Event::Message(client_id, message) => {
                    println!(
                        "Received a message from client #{}: {:?} which will be ignored",
                        client_id, message
                    );
                }
            }
        }
    });

    let subscriber_count = Arc::new(Mutex::new(0));
    let subscriber_count_clone = Arc::clone(&subscriber_count);

    // PROCESS both types of events synchronously
    let udp_receive_tx = tx.clone();
    std::thread::spawn(move || loop {
        // Handle next udp receive
        let (byte_count, client_addr) = &udp_socket.recv_from(&mut reusable_buffer).expect("Udp receive failed"); // <If this fails, let entire flow fail.
        let received_bytes = &mut reusable_buffer[..byte_count.to_owned()];

        // Pause until next udp message received
        let publish_message_or_none = read_next_publisher_data_message(
            &udp_socket,
            received_bytes,
            client_addr,
            &subscriber_count_clone,
        ).unwrap();

        // Now broad cast any publish message to all subscribers
        match publish_message_or_none {
            Some(publish_message) => {
                udp_receive_tx.send(JustRateAction::BroadCastMessageToAllClients(publish_message)).unwrap();
            }
            None => {}
        }
    });

    let mut websocket_clients: HashMap<u64, simple_websockets::Responder> = HashMap::new();
    loop {
        
        match rx.recv().unwrap() {
            JustRateAction::AddClient(client_id, responder) => {
                println!("Inserting client_id {}", client_id);
                websocket_clients.insert(client_id, responder);
                let mut sub_count = subscriber_count.lock().expect("Lock failed in counting up");
                *sub_count += 1;
                println!("Sub count: {}", sub_count);
            },
            JustRateAction::RemoveClient(client_id) => {
                websocket_clients.remove(&client_id);
                let mut sub_count = subscriber_count.lock().expect("lock failed when counting down");
                *sub_count -= 1;
                println!("Sub count: {}", sub_count);
            }
            JustRateAction::BroadCastMessageToAllClients(publish_message) => {
                let client_keys = websocket_clients.keys();
                for key in client_keys {
                    let client_responder = websocket_clients.get(key).unwrap();
                    let _message_was_sent = client_responder.send(
                        simple_websockets::Message::Text(publish_message.payload.clone()),
                    );
                }
            }
        };
    }
}

pub fn read_next_publisher_data_message(
    local_socket: &UdpSocket,
    received_bytes: &mut [u8],
    client_addr: &SocketAddr,
    subscriber_count: &Arc<Mutex<usize>>,
) -> std::io::Result<Option<PublisherMessage>> {
    // Validate bytes as valid utf8
    let valid_utf_string = match str::from_utf8(&received_bytes) {
        Ok(str) => str,
        Err(err) => {
            send_reply_to_client(
                "ERROR Invalid utf8 bytes. Error details: ".to_string() + &err.to_string(),
                &client_addr,
                &local_socket,
            )?;
            return Ok(None);
        }
    };

    // Extract first word and interpret as command
    let (command, payload_or_empty) = match valid_utf_string.find(' ') {
        Some(index) => {
            let (first_part, second_part) = valid_utf_string.split_at(index);
            (first_part, &second_part[1..]) // Skipping the first space in the beginning of the payload
        }
        None => (valid_utf_string, ""),
    };

    // Process message type and get data body if any
    match command {
        "DATA" => {
            if payload_or_empty.len() == 0 {
                send_reply_to_client(
                    "ERROR Empty payload received after a DATA command which is not valid."
                        .to_string(),
                    &client_addr,
                    &local_socket,
                )?;
            } else {
                return Ok(Some(PublisherMessage {
                    payload: payload_or_empty.to_string(),
                }));
            }
            return Ok(None);
            // Parse data to DataMessage.
            // => If success, add to channel sender
            // => If error, return error to client
        }
        "GET_SUBSCRIBER_COUNT" => {
            let has_payload = payload_or_empty.len() > 0;
            if has_payload {
                send_reply_to_client(
                    "ERROR Unexpected payload for command PING: ".to_string() + &payload_or_empty,
                    &client_addr,
                    &local_socket,
                )?;
            } else {
                let sub_count = subscriber_count.lock().unwrap();
                send_reply_to_client(
                    format!("SUBSCRIBER_COUNT {sub_count}").to_string(),
                    &client_addr,
                    &local_socket,
                )?;
            }
            return Ok(None);
        }
        "SUBSCRIBER_COUNT" | "ERROR" => {
            send_reply_to_client(
                "ERROR Client not allowed to send command ".to_string() + command,
                &client_addr,
                &local_socket,
            )?;
            return Ok(None);
        }
        _ => {
            print!("HIT HERE!");
            send_reply_to_client(
                "ERROR Unexpected protocol command: ".to_string() + command,
                &client_addr,
                &local_socket,
            )?;
            return Ok(None);
        }
    };
}

fn send_reply_to_client(
    message: String,
    client_addr: &SocketAddr,
    local_socket: &UdpSocket,
) -> std::io::Result<()> {
    local_socket.send_to(message.as_bytes(), client_addr)?;
    return Ok(());
}

pub struct PublisherMessage {
    pub payload: String,
}
