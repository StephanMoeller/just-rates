use std::net::{SocketAddr, UdpSocket};
use std::str;
use std::sync::mpsc::{Sender, Receiver};
use std::net::TcpListener;

const BUFFER_SIZE: usize = 10000;

// Protocol

pub fn create_publish_listener(local_socket: UdpSocket, data_message_sender: Sender<DataMessage>) -> std::io::Result<()> {
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
        let (command, payload_or_empty) = match valid_utf_string.find(' ') {
            Some(index) => {
                let (first_part, second_part) = valid_utf_string.split_at(index);
                (first_part, &second_part[1..]) // Skipping the first space in the beginning of the payload
            },
            None => (valid_utf_string, "")
        };

        // Process message type and get data body if any
        match command
        {
            "DATA" => {
                if payload_or_empty.len() == 0 {
                    send_reply_to_client("ERROR Empty payload received after a DATA command which is not valid.".to_string(), &client_addr, &local_socket)?;    
                }else{
                    let send_result = data_message_sender.send(DataMessage{ payload: payload_or_empty.to_string() });
                    match send_result {
                        Ok(()) => {},
                        Err(err) => {
                            println!("ERROR Internal error. {err}");
                            send_reply_to_client("ERROR Internal error. ".to_string() + &err.to_string(), &client_addr, &local_socket)?;    
                        }
                    }
                }
                continue;
                // Parse data to DataMessage.
                // => If success, add to channel sender
                // => If error, return error to client
            },
            "PING" => {
                let has_payload = payload_or_empty.len() > 0;
                if has_payload {
                    send_reply_to_client("ERROR Unexpected payload for command PING: ".to_string() + &payload_or_empty, &client_addr, &local_socket)?;    
                }else{
                    send_reply_to_client("PONG".to_string(), &client_addr, &local_socket)?;
                }
                
                continue;
            },
            "PONG" | "ERROR" => {
                send_reply_to_client("ERROR Client not allowed to send command ".to_string() + command, &client_addr, &local_socket)?;
                continue;
            }
            _ => {
                send_reply_to_client("ERROR Unexpected protocol command: ".to_string() + valid_utf_string, &client_addr, &local_socket)?;
                continue; // Return loop to the top
            }
        };
    }
}

pub fn create_consumer_endpoint(_local_tcp_listener: TcpListener, _data_message_reader: Receiver<DataMessage>){

}

fn send_reply_to_client(message: String, client_addr: &SocketAddr, local_socket: &UdpSocket) -> std::io::Result<()> {
    local_socket.send_to(message.as_bytes(), client_addr)?;
    return Ok(());
}

pub struct DataMessage{
    pub payload: String
}