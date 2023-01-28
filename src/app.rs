use std::net::{UdpSocket};
use std::str;

const BUFFER_SIZE: usize = 10000;
const CLIENT_CHECK_STATUS: &str = "CHECK STATUS";

const SERVER_STATUS_ACTIVE: &str = "STATUS ACTIVE"; // <= Returned if a consumer recently queried for data
const SERVER_STATUS_IDLE: &str = "STATUS IDLE"; // <= Returned if not client has requested data for some time. This way the producers can stop sending and save resources.

pub fn run(local_socket: UdpSocket) -> std::io::Result<()>
{
    let mut buffer: [u8; 10000] = [0; BUFFER_SIZE];
    
    // Create ready-to-use buffers containing standard replies for performance
    let status_active_reply_bytes: &[u8] = SERVER_STATUS_ACTIVE.as_bytes();
    let status_idle_reply_bytes: &[u8] = SERVER_STATUS_IDLE.as_bytes();

    loop
    {
        match local_socket.recv_from(&mut buffer) {
            Ok((byte_count, src)) => {
                let received_bytes = &mut buffer[..byte_count];

                match str::from_utf8(&received_bytes) {
                    Ok(str) => {
                        if str.eq(CLIENT_CHECK_STATUS) {
                            let is_active = true; // <= Base this on whether a client has polled for data recently
                            if is_active
                            {
                                local_socket.send(status_active_reply_bytes)?; // <= ? ensures fail of entire function if sending fails
                            }else{
                                local_socket.send(status_idle_reply_bytes)?; // <= ? ensures fail of entire function if sending fails
                            }
                            // Renew subscription
                        }else{
                            // Remove all expired subscribers (clients who havent sent a SUBSCRIBE message for a while)
                            // Send an UNSUBSCRIBED message to these clients for them to react in case they did not expect this
                            // TODO: Send to all subscribed clients
                            received_bytes.reverse();
                            local_socket.send_to(&received_bytes, &src)?; // <= ? ensures fail of entire function if sending fails
                        }
                    },
                    Err(err) => {
                        println!("Utf8 parse error: {err} when parsingreceived  bytes: {:?}", &received_bytes);
                    }
                }

                // Else:
                    // Iterate all subscribers
                        // If subscriber is recent => send received bytes to subscriber
                        // Else remove subscriber

            }
            Err(err_msg) => {
                println!("APP Error receiving: {err_msg}");
                break;
            },
        }
    }

    return Ok(());
}
