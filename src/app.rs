use std::net::{SocketAddr, UdpSocket};
use std::str;
use std::sync::mpsc::{Sender, Receiver};
use std::net::TcpListener;

const BUFFER_SIZE: usize = 10000;

// Protocol

pub fn create_publish_listener(local_socket: UdpSocket, _data_message_sender: Sender<DataMessage>) -> std::io::Result<()> {
    // Protocol messages - stores as byte array s
    
    let mut buffer: [u8; 10000] = [0; BUFFER_SIZE];

    loop {
        let (byte_count, client_addr) = local_socket.recv_from(&mut buffer)?; // <If this fails, let entire flow fail.
        let received_bytes = &mut buffer[..byte_count];
        
        // Validate bytes as valid utf8
        let valid_utf_string = match str::from_utf8(&received_bytes) {
            Ok(str) => str,
            Err(err) => {
                send_reply_to_client("ERROR Invalid utf8 bytes. Error details: ".to_string() + &err.to_string(), &client_addr, &local_socket)?;
                continue; // Return loop to the top
            }
        };

        // Extract first word and interpret as command
        let command = match valid_utf_string.find(' ') {
            Some(index) => valid_utf_string.split_at(index).0,
            None => valid_utf_string
        };

        // Process message type and get data body if any
        let _data_body = match command
        {
            "DATA" => valid_utf_string[4..].to_string(), // Gets all bytes after "DATA". This works on bytes but the word "DATA" contains only 1-byte characters and hence is safe to use here.
            "PING" => {
                send_reply_to_client("PONG".to_string(), &client_addr, &local_socket)?;
                continue;
            },
            _ => {
                send_reply_to_client("ERROR Unexpected protocol message: ".to_string() + valid_utf_string, &client_addr, &local_socket)?;
                continue; // Return loop to the top
            }
        };

        // TODO: Validate expected number of parts in data
        // Extract from, to and counter
        // Update some data structure with the new data
        // Done.
    }
}

pub fn create_consumer_endpoint(_local_tcp_listener: TcpListener, _data_message_reader: Receiver<DataMessage>){

}

fn send_reply_to_client(message: String, client_addr: &SocketAddr, local_socket: &UdpSocket) -> std::io::Result<()> {
    local_socket.send_to(message.as_bytes(), client_addr)?;
    return Ok(());
}

pub struct DataMessage{}