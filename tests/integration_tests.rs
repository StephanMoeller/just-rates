mod test_utils;

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use crate::test_utils::*;
    
    #[test]
    fn send_receive_test()
    {
        let (server_thread, server_addr) = util_start_server();
        
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let reply = util_send_and_receive_internal(&client, server_addr, "HELLO");
        assert_eq!(reply.as_str(), "OLLEH");
        assert_eq!(false, server_thread.is_finished());
    }

    
}