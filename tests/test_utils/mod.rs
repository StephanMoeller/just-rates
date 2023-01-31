
use std::str;
use std::{
    net::UdpSocket
};

pub fn create_socket_with_receive_timeout() -> UdpSocket{
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap(); // With timeout to ensure fail if nothing is received when expected
    return socket;
}

pub fn receive_string(socket: &UdpSocket) -> String{
    let mut buffer: [u8; 1000] = [0; 1000];
    let (amt, _src) = socket.recv_from(&mut buffer).unwrap();
    let reply = str::from_utf8(&buffer[..amt]).unwrap();
    return reply.to_string();
}

pub fn ensure_nothing_to_receive(socket: &UdpSocket)
{
    let mut buffer: [u8; 1000] = [0; 1000];
    match socket.recv_from(&mut buffer)
    {
        Ok((_size, _src)) => panic!("Did not expect to receive anything here"),
        Err(_) => {} // Expected to fail
    }
}