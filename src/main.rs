use simple_websockets::{Event, Responder};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
extern crate ctrlc;
mod app;

fn main() -> std::io::Result<()> {
    // This channel is the glue between udp listener and tcp sender
    let publisher_channel: (
        Sender<app::PublisherMessage>,
        Receiver<app::PublisherMessage>,
    ) = mpsc::channel();
    let websocket_event_channel: (Sender<Event>, Receiver<Event>) = mpsc::channel();

    let udp_addr = "127.0.0.1:8343";
    let websocket_port = 8081;
    let udp_socket = std::net::UdpSocket::bind(&String::from(udp_addr)).expect(
        ("Error binding udp on local address ".to_string() + &udp_addr.to_string()).as_str(),
    );
    let websocket_event_hub = simple_websockets::launch(8081).expect(
        ("failed to listen on websocket port ".to_string() + &websocket_port.to_string()).as_str(),
    );

    // Incoming udp => publisher channel
    std::thread::spawn(move || loop {
        let publish_result = app::read_next_publisher_data_message(&udp_socket).unwrap();
        match publish_result
        {
            Some(publish_message) => {
                publisher_channel.0.send(publish_message).unwrap();
            },
            None => {}
        }
    });

    // Websocket event => websocket channel
    std::thread::spawn(move || loop {
        let event = websocket_event_hub.poll_event();
        websocket_event_channel.0.send(event).unwrap();
    });

    let mut websocket_clients: HashMap<u64, Responder> = HashMap::new();
    loop {
        // Websocket channel => adjust client dictionary
        let mut next_websocket_event = websocket_event_channel.1.try_recv();
        while next_websocket_event.is_ok() {
            match next_websocket_event.unwrap() {
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
            }
            next_websocket_event = websocket_event_channel.1.try_recv();
        }

        // Publisher channel => send to all web socket clients
        let mut next_publisher_event = publisher_channel.1.try_recv();
        while next_publisher_event.is_ok() {
            // TODO: Send to all websocket clients
            next_publisher_event = publisher_channel.1.try_recv();
        }
    }
}