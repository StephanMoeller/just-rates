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
    #[case("INVALID MESSAGE HERE")]
    #[case("PLEASE_SEND")]                  // Expected to only be sent by server to client, no the other way around
    #[case("PLEASE_SEND with more data")]   // Expected to only be sent by server to client, no the other way around
    #[case("PLEASE_SLEEP")]                 // Expected to only be sent by server to client, no the other way around
    #[case("PLEASE_SLEEP with more data")]  // Expected to only be sent by server to client, no the other way around
    #[case("ERROR")]                        // Expected to only be sent by server to client, no the other way around
    #[case("ERROR With more data")]         // Expected to only be sent by server to client, no the other way around
    fn receive_invalid_message_expect_error_returned_test(#[case] invalid_message_to_send: &str)
    {
        let (data_message_sender, _data_message_receiver): (Sender<DataMessage>, Receiver<DataMessage>) = mpsc::channel();

        let (server_thread, server_addr) = util_start_server(data_message_sender);
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let reply = util_send_and_receive_internal(&client, server_addr, invalid_message_to_send);
        assert_eq!(reply.as_str(), "ERROR Unexpected protocol message: ".to_string() + invalid_message_to_send);

        // Ensure nothing added to the reader
        match _data_message_receiver.recv_timeout(std::time::Duration::from_millis(200)){
            Ok(_) => {panic!("Expected timeout as a sign that nothing was added to the channel based on an invalid message received")},
            Err(_) => {}
        }
        assert_eq!(false, server_thread.is_finished());
    }

    
}