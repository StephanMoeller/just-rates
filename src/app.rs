use std::net::{SocketAddr, UdpSocket};
use std::str;

const BUFFER_SIZE: usize = 10000;

// Protocol
pub fn read_next_publisher_data_message(local_socket: &UdpSocket) -> std::io::Result<Option<PublisherMessage>> {
    // TODO: Solve reuse of this buffer
    let mut buffer: [u8; 10000] = [0; BUFFER_SIZE];

    let (byte_count, client_addr) = local_socket.recv_from(&mut buffer)?; // <If this fails, let entire flow fail.
        let received_bytes = &mut buffer[..byte_count];
        
        // Validate bytes as valid utf8
        let valid_utf_string = match str::from_utf8(&received_bytes) {
            Ok(str) => str,
            Err(err) => {
                send_reply_to_client("ERROR Invalid utf8 bytes. Error details: ".to_string() + &err.to_string(), &client_addr, &local_socket)?;
                return Ok(None);
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
                    return Ok(Some(PublisherMessage{ payload: payload_or_empty.to_string() }));
                }
                return Ok(None);
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
                return Ok(None);
            },
            "PONG" | "ERROR" => {
                send_reply_to_client("ERROR Client not allowed to send command ".to_string() + command, &client_addr, &local_socket)?;
                return Ok(None);
            }
            _ => {
                send_reply_to_client("ERROR Unexpected protocol command: ".to_string() + valid_utf_string, &client_addr, &local_socket)?;
                return Ok(None);
            }
        };
}

fn send_reply_to_client(message: String, client_addr: &SocketAddr, local_socket: &UdpSocket) -> std::io::Result<()> {
    local_socket.send_to(message.as_bytes(), client_addr)?;
    return Ok(());
}

pub struct PublisherMessage{
    pub payload: String
}

