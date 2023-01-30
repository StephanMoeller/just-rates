mod test_utils;

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use std::sync::mpsc::{Sender, Receiver};
    use std::sync::mpsc;
    use rust_just_rates::app::{PublisherMessage};
    use rstest::rstest;

    #[rstest]
    #[case("INVALID MESSAGE HERE", "ERROR Unexpected protocol command: INVALID MESSAGE HERE")]
    #[case("ERROR", "ERROR Client not allowed to send command ERROR")]                        
    #[case("ERROR With more data", "ERROR Client not allowed to send command ERROR")]         
    #[case("PONG", "ERROR Client not allowed to send command PONG")]                         
    #[case("PONG With more data", "ERROR Client not allowed to send command PONG")]          
    #[case("PING With more data", "ERROR Unexpected payload for command PING: With more data")]          
    #[case("Ping", "ERROR Unexpected protocol command: Ping")]                         
    #[case("ping", "ERROR Unexpected protocol command: ping")]    
    #[case("DATA", "ERROR Empty payload received after a DATA command which is not valid.")]                                       
    #[case("Data", "ERROR Unexpected protocol command: Data")]                         
    #[case("DATA ", "ERROR Empty payload received after a DATA command which is not valid.")]           
    fn invalid_message_expect_error_returned_test(#[case] invalid_message_to_send: &str, #[case] expected_message_to_receive: &str)
    {
        let (data_message_sender, _data_message_receiver): (Sender<PublisherMessage>, Receiver<PublisherMessage>) = mpsc::channel();
        let (server_addr, client_socket) = start_server_and_create_client_socket(data_message_sender);

        let reply = send_and_receive_internal(&client_socket, server_addr, invalid_message_to_send.as_bytes());
        assert_eq!(reply.as_str(), expected_message_to_receive);

        // Ensure nothing added to the reader
        assert_channel_empty(_data_message_receiver);
    }

    #[test]
    fn invalid_utf8_characters_expect_error_returned_test()
    {
        let (data_message_sender, _data_message_receiver): (Sender<PublisherMessage>, Receiver<PublisherMessage>) = mpsc::channel();

        let (server_addr, client_socket) = start_server_and_create_client_socket(data_message_sender);
        
        let mut invalid_utf8_bytes = "DATA Something more".as_bytes().to_owned();
        invalid_utf8_bytes[6] = 147; // Invalid utf8-character
        invalid_utf8_bytes[7] = 147; // Invalid utf8-character
        invalid_utf8_bytes[8] = 147; // Invalid utf8-character
        
        let reply = send_and_receive_internal(&client_socket, server_addr, &invalid_utf8_bytes);
        assert_eq!("ERROR Invalid utf8 bytes. Error details: invalid utf-8 sequence of 1 bytes from index 6", reply.as_str());
        assert_channel_empty(_data_message_receiver);
    }

    #[test]
    fn ping_expect_pong_returned_test()
    {
        let (data_message_sender, _data_message_receiver): (Sender<PublisherMessage>, Receiver<PublisherMessage>) = mpsc::channel();

        let (server_addr, client_socket) = start_server_and_create_client_socket(data_message_sender);
        
        let reply = send_and_receive_internal(&client_socket, server_addr, "PING".as_bytes());
        assert_eq!("PONG", reply.as_str());
        assert_channel_empty(_data_message_receiver);
    }

    #[test]
    fn data_expect_message_ended_up_in_channel_test()
    {
        let (data_message_sender, data_message_receiver): (Sender<PublisherMessage>, Receiver<PublisherMessage>) = mpsc::channel();
        let (server_addr, client_socket) = start_server_and_create_client_socket(data_message_sender);
        _ = &client_socket.send_to("DATA This is the data provided \n in multiple \n\r lines".as_bytes(), server_addr).unwrap();
        _ = &client_socket.send_to("DATA This is another message".as_bytes(), server_addr).unwrap();

        assert_eq!("This is the data provided \n in multiple \n\r lines", data_message_receiver.recv_timeout(std::time::Duration::from_secs(1)).unwrap().payload.as_str());
        assert_eq!("This is another message", data_message_receiver.recv_timeout(std::time::Duration::from_secs(1)).unwrap().payload.as_str());

        
        // Sleep and ensure nothing to be received on client socket
        let mut buffer: [u8; 10] = [0; 10];
        std::thread::sleep(std::time::Duration::from_millis(500));
        match &client_socket.recv_from(&mut buffer) {
            Ok(_) => panic!("Unexpected message received"),
            Err(_) => {} // Expect error do occur as sending DATA should not trigger the server to reply with any message
        }
    }
}