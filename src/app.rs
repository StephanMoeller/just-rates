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
        let (byte_count, client) = local_socket.recv_from(&mut buffer)?; // <If this fails, let entire flow fail.
        let received_bytes = &mut buffer[..byte_count];
        
        // Validate bytes as valid utf8
        let valid_utf_string = match str::from_utf8(&received_bytes) {
            Ok(str) => str,
            Err(err) => {
                reply_with_error("Invalid utf8 bytes. Error details: ".to_string() + &err.to_string(), &client, &local_socket)?;
                continue; // Return loop to the top
            }
        };

        let first_word = "TODO...";

        // Process message type and get data body if any
        let _data_body = match first_word
        {
            "DATA" => valid_utf_string[4..].to_string(), // Gets all bytes after "DATA". This works on bytes but the word "DATA" contains only 1-byte characters and hence is safe to use here.
            "SHOULD_I_SEND" => {
                // Reply with PLEASE_SEND or PLEASE_SLEEP
                continue; // Return loop to the top
            },
            _ => {
                reply_with_error("Unexpected protocol message: ".to_string() + valid_utf_string, &client, &local_socket)?;
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

fn reply_with_error(error_details: String, client_addr: &SocketAddr, local_socket: &UdpSocket) -> std::io::Result<()> {
    let mut error_msg = "ERROR ".to_string();
    error_msg.push_str(&error_details.as_str());
    local_socket.send_to(error_msg.as_bytes(), client_addr)?;
    return Ok(());
}

pub struct DataMessage{}