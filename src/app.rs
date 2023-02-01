
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub fn run(
    udp_socket: UdpSocket,
    websocket_event_hub: simple_websockets::EventHub,
) -> std::io::Result<()> {
    let mut reusable_buffer: [u8; 10000] = [0; 10000];
    let (tx, rx): (
        Sender<simple_websockets::Event>,
        Receiver<simple_websockets::Event>,
    ) = mpsc::channel();

    //  Websocket events => Channel
    let websocket_tx = tx.clone();
    std::thread::spawn(move || loop {
        loop {
            let event = websocket_event_hub.poll_event();
            websocket_tx.send(event).unwrap();
        }
    });

    // PROCESS both types of events synchronously
    let mut websocket_clients: HashMap<u64, simple_websockets::Responder> = HashMap::new();

    loop {
        // Receive next UDP message
        let (byte_count, client_addr) = &udp_socket.recv_from(&mut reusable_buffer)?; // <If this fails, let entire flow fail.
        let received_bytes = &mut reusable_buffer[..byte_count.to_owned()];

        // Before processing, handle all websocket events first to be sure to be in sync
        let mut rx_result = rx.try_recv();
        while rx_result.is_ok() {
            match rx_result.unwrap() {
                simple_websockets::Event::Connect(client_id, responder) => {
                    websocket_clients.insert(client_id, responder);
                    println!("A client connected with id #{}", client_id);
                }
                simple_websockets::Event::Disconnect(client_id) => {
                    websocket_clients.remove(&client_id);
                    println!("Client #{} disconnected.", client_id);
                }
                simple_websockets::Event::Message(client_id, message) => {
                    println!(
                        "Received a message from client #{}: {:?} which will be ignored",
                        client_id, message
                    );
                }
            }
            rx_result = rx.try_recv();
        }

        // Pause until next udp message received
        let publish_message_or_none = read_next_publisher_data_message(
            &udp_socket,
            received_bytes,
            client_addr,
            websocket_clients.len(),
        )?;

        // Now broad cast any publish message to all subscribers
        match publish_message_or_none {
            Some(publish_message) => {
                for (_client_id, client_responder) in &websocket_clients {
                    let _message_was_sent = client_responder.send(
                        simple_websockets::Message::Text(publish_message.payload.clone()),
                    );
                }
            }
            None => {}
        }
    }
}

pub fn read_next_publisher_data_message(
    local_socket: &UdpSocket,
    received_bytes: &mut [u8],
    client_addr: &SocketAddr,
    subscriber_count: usize,
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
                send_reply_to_client(
                    format!("SUBSCRIBER_COUNT {subscriber_count}").to_string(),
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
