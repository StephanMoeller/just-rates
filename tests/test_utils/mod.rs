
use rust_just_rates::app;
use std::str;
use std::{
    net::{SocketAddr, UdpSocket},
    thread::JoinHandle,
};

pub fn util_send_and_receive_internal(
    client: &UdpSocket,
    server_addr: SocketAddr,
    data_to_send: &str,
) -> String {
    let bytes = data_to_send.as_bytes(); // Gives utf8 bytes
    client.send_to(bytes, server_addr).unwrap();

    let mut buffer: [u8; 1000] = [0; 1000];
    let (amt, _src) = client.recv_from(&mut buffer).unwrap();
    let str_value = str::from_utf8(&buffer[..amt]).unwrap();
    return String::from(str_value);
}

pub fn util_start_server() -> (JoinHandle<()>, SocketAddr) {
    let server_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let server_addr = server_socket.local_addr().unwrap();
    println!("SERVER ADDRESS: {}", server_addr);
    let app_thread = std::thread::spawn(move || {
        app::run(server_socket).unwrap();
    });
    return (app_thread, server_addr);
}
