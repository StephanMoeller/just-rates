mod test_utils;

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use rstest::rstest;
    use rust_just_rates::app;
    use std::str;

    #[rstest]
    #[case("INVALID MESSAGE HERE", "ERROR Unexpected protocol command: INVALID")]
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
        // Init
        let (client_socket, server_socket, mut reusable_buffer) = (create_socket_with_receive_timeout(), create_socket_with_receive_timeout(), [0; 10000]);
        
        // Execute
        client_socket.send_to(invalid_message_to_send.as_bytes(), server_socket.local_addr().unwrap()).unwrap();
        let server_process_result = app::read_next_publisher_data_message(&server_socket, &mut reusable_buffer).unwrap();
        
        // Assert no new data message
        assert_eq!(true, server_process_result.is_none());

        // Assert expected reply sent to client
        let reply = receive_string(&client_socket);
        assert_eq!(true, server_process_result.is_none()); // Expect no data message returned
        assert_eq!(expected_message_to_receive, reply);
    }

    #[test]
    fn invalid_utf8_characters_expect_error_returned_test()
    {
        // Init
        let (client_socket, server_socket, mut reusable_buffer) = (create_socket_with_receive_timeout(), create_socket_with_receive_timeout(), [0; 10000]);
        let mut invalid_utf8_bytes = "DATA Something more".as_bytes().to_owned();
        invalid_utf8_bytes[6] = 147; // Invalid utf8-character
        invalid_utf8_bytes[7] = 147; // Invalid utf8-character
        invalid_utf8_bytes[8] = 147; // Invalid utf8-character

        // Execute
        client_socket.send_to(&invalid_utf8_bytes, server_socket.local_addr().unwrap()).unwrap();
        let server_process_result = app::read_next_publisher_data_message(&server_socket, &mut reusable_buffer).unwrap();
        
        // Assert no new data message
        assert_eq!(true, server_process_result.is_none());

        // Assert expected reply sent to client
        let reply = receive_string(&client_socket);
        assert_eq!(true, server_process_result.is_none()); // Expect no data message returned
        assert_eq!("ERROR Invalid utf8 bytes. Error details: invalid utf-8 sequence of 1 bytes from index 6", reply);
    }

    #[test]
    fn ping_expect_pong_returned_test()
    {
        // Init
        let (client_socket, server_socket, mut reusable_buffer) = (create_socket_with_receive_timeout(), create_socket_with_receive_timeout(), [0; 10000]);
        
        // Execute
        client_socket.send_to("PING".as_bytes(), server_socket.local_addr().unwrap()).unwrap();
        let server_process_result = app::read_next_publisher_data_message(&server_socket, &mut reusable_buffer).unwrap();

        // Assert no new data message
        assert_eq!(true, server_process_result.is_none());

        // Assert expected reply sent to client
        let reply = receive_string(&client_socket);
        assert_eq!("PONG", reply.as_str());
    }

    #[test]
    fn data_expect_message_ended_up_in_channel_test()
    {
        // Init
        let (client_socket, server_socket, mut reusable_buffer) = (create_socket_with_receive_timeout(), create_socket_with_receive_timeout(), [0; 10000]);
        
        // Execute
        client_socket.send_to("DATA This is the data provided \n in multiple \n\r lines".as_bytes(), server_socket.local_addr().unwrap()).unwrap();
        let server_process_result = app::read_next_publisher_data_message(&server_socket, &mut reusable_buffer).unwrap();

        // Assert data message returned
        assert_eq!(true, server_process_result.is_some());
        assert_eq!("This is the data provided \n in multiple \n\r lines", server_process_result.unwrap().payload.as_str());
        
        // Assert no reply sent on data messages
        ensure_nothing_to_receive(&client_socket);
    }
}