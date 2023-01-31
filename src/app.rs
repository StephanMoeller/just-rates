use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub fn run(publisher_udp_port: i32, websocket_tcp_port: i32) -> std::io::Result<()> {
    enum ChannelMessage {
        Event(simple_websockets::Event),
        PublisherMessage(PublisherMessage),
    }
    let mut reusable_buffer: [u8; 10000] = [0; 10000];
    let (tx, rx): (Sender<ChannelMessage>, Receiver<ChannelMessage>) = mpsc::channel();

    // SETUP udp listening socket and websocket server
    let udp_addr = format!("127.0.0.1:{publisher_udp_port}");
    let udp_socket = std::net::UdpSocket::bind(&String::from(&udp_addr))
        .expect(format!("Error binding udp on local address {udp_addr}").as_str());
    let websocket_event_hub = simple_websockets::launch(8081)
        .expect(format!("failed to listen on websocket port {websocket_tcp_port}").as_str());

    // START receiving and add to channel
    //  Publisher message => Channel
    let publisher_tx = tx.clone();
    std::thread::spawn(move || loop {
        loop {
            let publish_result = read_next_publisher_data_message(&udp_socket, &mut reusable_buffer).unwrap();
            if publish_result.is_some() {
                let channel_msg = ChannelMessage::PublisherMessage(publish_result.unwrap());
                publisher_tx.send(channel_msg).unwrap();
            }
        }
    });

    //  Websocket events => Channel
    let websocket_tx = tx.clone();
    std::thread::spawn(move || loop {
        loop {
            let event = websocket_event_hub.poll_event();
            let channel_msg = ChannelMessage::Event(event);
            websocket_tx.send(channel_msg).unwrap();
        }
    });

    // PROCESS both types of events synchronously
    let mut websocket_clients: HashMap<u64, simple_websockets::Responder> = HashMap::new();
    loop {
        match rx.recv().unwrap() {
            ChannelMessage::PublisherMessage(publisher_message) => {
                println!("Processing message {}", publisher_message.payload);
                // Send received message from publisher to all connected websocket clients
                for (_client_id, client_responder) in &websocket_clients {
                    let _message_was_sent = client_responder.send(
                        simple_websockets::Message::Text(publisher_message.payload.clone()),
                    );
                }
            }
            ChannelMessage::Event(event) => match event {
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
            },
        }
    }
}

pub fn read_next_publisher_data_message(
    local_socket: &UdpSocket,
    reusable_buffer: &mut [u8],
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
        "PING" => {
            let has_payload = payload_or_empty.len() > 0;
            if has_payload {
                send_reply_to_client(
                    "ERROR Unexpected payload for command PING: ".to_string() + &payload_or_empty,
                    &client_addr,
                    &local_socket,
                )?;
            } else {
                send_reply_to_client("PONG".to_string(), &client_addr, &local_socket)?;
            }
            return Ok(None);
        }
        "PONG" | "ERROR" => {
            send_reply_to_client(
                "ERROR Client not allowed to send command ".to_string() + command,
                &client_addr,
                &local_socket,
            )?;
            return Ok(None);
        }
        _ => {
            send_reply_to_client(
                "ERROR Unexpected protocol command: ".to_string() + valid_utf_string,
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
