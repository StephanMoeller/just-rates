
use std::net::{UdpSocket, TcpListener};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

extern crate ctrlc;
mod app;

fn main() -> std::io::Result<()> {

    // This channel is the glue between udp listener and tcp sender
    let (data_message_sender, data_message_receiver): (Sender<app::DataMessage>, Receiver<app::DataMessage>) = mpsc::channel();
    let udp_addr = "127.0.0.1:8343";
    let tcp_addr = "127.0.0.1:8081";

    let (udp_socket, tcp_listener) = setup_udp_socket_and_tcp_listener(udp_addr, tcp_addr);
    let publisher_thread = std::thread::spawn(move || {
        app::create_publish_listener(udp_socket, data_message_sender).unwrap();
    });

    let consumer_thread = std::thread::spawn(move || {       
        app::create_consumer_endpoint(tcp_listener, data_message_receiver);
    });
    
    publisher_thread.join().unwrap();
    consumer_thread.join().unwrap();

    return Ok(());
}

fn setup_udp_socket_and_tcp_listener(udp_addr: &str, tcp_addr: &str) -> (UdpSocket, TcpListener){

    // Create local udp and tcp stuff in main thread before moving to threads to break early on main thread if any of these two fails
    let local_socket = match std::net::UdpSocket::bind(&String::from(udp_addr)) {
        Ok(socket) => socket,
        Err(msg) => panic!("Error binding udp on local address {udp_addr}: {msg}")
    };

    let local_tcp_listener = match std::net::TcpListener::bind(&String::from(tcp_addr)) {
        Ok(listener) => listener,
        Err(msg) => panic!("Error binding tcp listener on local address {tcp_addr}: {msg}")
    };

    return (local_socket, local_tcp_listener);
}