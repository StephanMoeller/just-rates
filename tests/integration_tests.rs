mod test_utils;


#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use crate::test_utils::*;
    use std::sync::mpsc::{Sender, Receiver};
    use std::sync::mpsc;
    use rust_just_rates::app::{DataMessage};

    #[test]
    fn send_receive_test()
    {
        let (data_message_sender, _data_message_receiver): (Sender<DataMessage>, Receiver<DataMessage>) = mpsc::channel();

        let (server_thread, server_addr) = util_start_server(data_message_sender);
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let reply = util_send_and_receive_internal(&client, server_addr, "HELLO");
        assert_eq!(reply.as_str(), "OLLEH");
        assert_eq!(false, server_thread.is_finished());
    }

    
}