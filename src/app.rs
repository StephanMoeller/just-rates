use std::net::{UdpSocket};

const BUFFER_SIZE: usize = 10000;

pub fn run(local_address: &String) -> std::io::Result<()>
{
    let socket = UdpSocket::bind(local_address)?;
    let mut buffer: [u8; 10000] = [0; BUFFER_SIZE];
    
    loop
    {
        println!("Thread waiting for new message...");
        
        match socket.recv_from(&mut buffer) {
            Ok(val) => {
                let (byte_count, src) = val;
                let received_bytes = &mut buffer[..byte_count];
                println!("Bytes received: {:?}", received_bytes);
                received_bytes.reverse();
                println!("Replying with reversed byte stream: {:?}", received_bytes);
                match socket.send_to(&received_bytes, &src){
                    Ok(_) => {
                        println!("Sent successfully. Repeating loop.");
                    },
                    Err(err_msg) => {
                        println!("Error sending: {err_msg}");
                        break;
                    },
                }
            }
            Err(err_msg) => {
                println!("Error receiving: {err_msg}");
                break;
            },
        }
    }

    return Ok(());
}
