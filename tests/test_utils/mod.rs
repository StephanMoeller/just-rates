
use rust_just_rates::app;
use std::sync::mpsc::{Sender, Receiver};
use rust_just_rates::app::{DataMessage};
use std::str;
use std::{
    net::{SocketAddr, UdpSocket}
};

pub fn send_and_receive_internal(
    client: &UdpSocket,
    server_addr: SocketAddr,
    data_to_send: &[u8],
) -> String {
    client.send_to(data_to_send, server_addr).unwrap();
    
    let mut buffer: [u8; 1000] = [0; 1000];
    let (amt, _src) = client.recv_from(&mut buffer).unwrap();
    let str_value = str::from_utf8(&buffer[..amt]).unwrap();
    return String::from(str_value);
}

pub fn start_server_and_create_client_socket(data_message_sender: Sender<DataMessage>) -> (SocketAddr, UdpSocket) {
    
    let server_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let server_addr = server_socket.local_addr().unwrap();
    println!("SERVER ADDRESS: {}", server_addr);
    let _app_thread = std::thread::spawn(move || {
        app::create_publish_listener(server_socket, data_message_sender).unwrap();
    });
    let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    client_socket.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap();
    return (server_addr, client_socket);
}

pub fn assert_channel_empty<T>(reader: Receiver<T>) {
    match reader.try_recv(){
        Ok(_) => {panic!("Expected timeout as a sign that nothing was added to the channel based on an invalid message received")},
        Err(_) => {}
    }
}
