use std::net::{SocketAddr, UdpSocket};
use std::str;

const BUFFER_SIZE: usize = 10000;

// Protocol

pub fn run(local_socket: UdpSocket) -> std::io::Result<()> {
    // Protocol messages - stores as byte array s
    
    let mut buffer: [u8; 10000] = [0; BUFFER_SIZE];

    loop {
        let (byte_count, client) = local_socket.recv_from(&mut buffer)?; // <If this fails, let entire flow fail.
        let received_bytes = &mut buffer[..byte_count];
        
        // Validate bytes as valid utf8
        let valid_utf_string = match str::from_utf8(&received_bytes) {
            Ok(str) => str,
            Err(err) => {
                reply_with_error("Invalid utf8 bytes. Error details: ".to_string() + &err.to_string(), &client, &local_socket);
                continue; // Return loop to the top
            }
        };

        let first_word = "TODO...";

        // Process message type and get data body if any
        let data = match first_word
        {
            "DATA" => valid_utf_string[4..].to_string(), // Gets all bytes after "DATA". This works on bytes but the word "DATA" contains only 1-byte characters and hence is safe to use here.
            "SHOULD_I_SEND" => {
                // Reply with PLEASE_SEND or PLEASE_SLEEP
                continue;
            },
            _ => {
                reply_with_error("Unexpected protocol message starting with ".to_string() + first_word, &client, &local_socket);
                continue;
            }
        };
    }

    return Ok(());
}

fn reply_with_error(error_details: String, client: &SocketAddr, local_socket: &UdpSocket) {
    // TODO: Reuse some byte buffer
}