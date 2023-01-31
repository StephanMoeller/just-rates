use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub fn run(publisher_udp_port: i32, websocket_tcp_port: i32) -> std::io::Result<()> {
    let mut reusable_buffer: [u8; 10000] = [0; 10000];
    let (tx, rx): (
        Sender<simple_websockets::Event>,
        Receiver<simple_websockets::Event>,
    ) = mpsc::channel();

    // SETUP udp listening socket and websocket server
    let udp_addr = format!("127.0.0.1:{publisher_udp_port}");
    let udp_socket = std::net::UdpSocket::bind(&String::from(&udp_addr))
        .expect(format!("Error binding udp on local address {udp_addr}").as_str());
    let websocket_event_hub = simple_websockets::launch(8081)
        .expect(format!("failed to listen on websocket port {websocket_tcp_port}").as_str());

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
    let mut rate_counter = 0;
    let mut last_reset_time = std::time::Instant::now();

    loop {
        rate_counter += 1;
        let duration_secs = last_reset_time.elapsed().as_millis();
        if duration_secs > 1000 {
            println!("{}/sec", rate_counter);
            last_reset_time = std::time::Instant::now();
            rate_counter = 0;
        }
        // Pause until next udp message received
        let publish_message_or_none = read_next_publisher_data_message(
            &udp_socket,
            &mut reusable_buffer,
            websocket_clients.len(),
        )?;

        // Before processing the publish message, update the websocket list with any recent events
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
    reusable_buffer: &mut [u8],
    subscriber_count: usize,
) -> std::io::Result<Option<PublisherMessage>> {
    // Receive next udp datagram
    let (byte_count, client_addr) = local_socket.recv_from(reusable_buffer)?; // <If this fails, let entire flow fail.
    let received_bytes = &mut reusable_buffer[..byte_count];

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
