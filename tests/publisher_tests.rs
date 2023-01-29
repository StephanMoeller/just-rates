mod test_utils;

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use crate::test_utils::*;
    use std::sync::mpsc::{Sender, Receiver};
    use std::sync::mpsc;
    use rust_just_rates::app::{DataMessage};
    use rstest::rstest;

    #[rstest]
    #[case("INVALID MESSAGE HERE", "ERROR Unexpected protocol message: INVALID MESSAGE HERE")]
    #[case("ERROR", "ERROR Client not allowed to send command ERROR")]                        
    #[case("ERROR With more data", "ERROR Client not allowed to send command ERROR")]         
    #[case("PONG", "ERROR Client not allowed to send command PONG")]                         
    #[case("PONG With more data", "ERROR Client not allowed to send command PONG")]          
    #[case("PING With more data", "ERROR Unexpected payload for command PING: With more data")]          
    #[case("Ping", "ERROR Unexpected protocol message: Ping")]                         
    #[case("ping", "ERROR Unexpected protocol message: ping")]                         
    fn receive_invalid_message_expect_error_returned_test(#[case] invalid_message_to_send: &str, #[case] expected_message_to_receive: &str)
    {
        let (data_message_sender, _data_message_receiver): (Sender<DataMessage>, Receiver<DataMessage>) = mpsc::channel();

        let server_addr = start_server(data_message_sender);
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let reply = send_and_receive_internal(&client, server_addr, invalid_message_to_send.as_bytes());
        assert_eq!(reply.as_str(), expected_message_to_receive);

        // Ensure nothing added to the reader
        assert_channel_empty(_data_message_receiver);
    }

    #[test]
    fn receive_invalid_utf8_characters_expect_error_returned_test()
    {
        let (data_message_sender, _data_message_receiver): (Sender<DataMessage>, Receiver<DataMessage>) = mpsc::channel();

        let server_addr = start_server(data_message_sender);
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let mut invalid_utf8_bytes = "DATA Something more".as_bytes().to_owned();
        invalid_utf8_bytes[6] = 147; // Invalid utf8-character
        invalid_utf8_bytes[7] = 147; // Invalid utf8-character
        invalid_utf8_bytes[8] = 147; // Invalid utf8-character
        
        let reply = send_and_receive_internal(&client, server_addr, &invalid_utf8_bytes);
        assert_eq!("ERROR Invalid utf8 bytes. Error details: invalid utf-8 sequence of 1 bytes from index 6", reply.as_str());
        assert_channel_empty(_data_message_receiver);
    }

    #[test]
    fn receive_ping_expect_pong_returned_test()
    {
        let (data_message_sender, _data_message_receiver): (Sender<DataMessage>, Receiver<DataMessage>) = mpsc::channel();

        let server_addr = start_server(data_message_sender);
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let reply = send_and_receive_internal(&client, server_addr, "PING".as_bytes());
        assert_eq!("PONG", reply.as_str());
        assert_channel_empty(_data_message_receiver);
    }
}