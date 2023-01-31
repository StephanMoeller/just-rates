use simple_websockets::{Event, Responder};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
extern crate ctrlc;
mod app;

fn main() -> std::io::Result<()> {
    enum ChannelMessage {
        Event(Event),
        PublisherMessage(app::PublisherMessage),
    }

    let (tx, rx): (Sender<ChannelMessage>, Receiver<ChannelMessage>) = mpsc::channel();

    let udp_addr = "127.0.0.1:8343";
    let websocket_port = 8081;
    let udp_socket = std::net::UdpSocket::bind(&String::from(udp_addr)).expect(
        ("Error binding udp on local address ".to_string() + &udp_addr.to_string()).as_str(),
    );
    let websocket_event_hub = simple_websockets::launch(8081).expect(
        ("failed to listen on websocket port ".to_string() + &websocket_port.to_string()).as_str(),
    );

    // Publisher message => Channel
    let publisher_tx = tx.clone();
    std::thread::spawn(move || loop {
        loop {
            let publish_result = app::read_next_publisher_data_message(&udp_socket).unwrap();
            if publish_result.is_some() {
                let channel_msg = ChannelMessage::PublisherMessage(publish_result.unwrap());
                publisher_tx.send(channel_msg).unwrap();
            }
        }
    });

    // Websocket events => Channel
    let websocket_tx = tx.clone();
    std::thread::spawn(move || loop {
        loop {
            let event = websocket_event_hub.poll_event();
            let channel_msg = ChannelMessage::Event(event);
            websocket_tx.send(channel_msg).unwrap();
        }
    });

    // Process both types of events synchronously
    let mut websocket_clients: HashMap<u64, Responder> = HashMap::new();
    loop {
        match rx.recv().unwrap() {
            ChannelMessage::PublisherMessage(msg) => {
                // TODO: Send to all websocket clients
            },
            ChannelMessage::Event(event) => match event {
                Event::Connect(client_id, responder) => {
                    websocket_clients.insert(client_id, responder);
                    println!("A client connected with id #{}", client_id);
                }
                Event::Disconnect(client_id) => {
                    websocket_clients.remove(&client_id);
                    println!("Client #{} disconnected.", client_id);
                }
                Event::Message(client_id, message) => {
                    println!(
                        "Received a message from client #{}: {:?} which will be ignored",
                        client_id, message
                    );
                }
            },
        }
    }
}
