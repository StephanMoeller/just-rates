// mod test_utils;

// #[cfg(test)]
// mod tests {
//     use crate::test_utils::*;
//     use rstest::rstest;
//     use rust_just_rates::app;
//     use std::str;



//     #[test]
//     fn ping_expect_pong_returned_test()
//     {
//         // Init
//         let (client_socket, server_socket, mut reusable_buffer) = (create_socket_with_receive_timeout(), create_socket_with_receive_timeout(), [0; 10000]);
        
//         // Execute
//         client_socket.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), server_socket.local_addr().unwrap()).unwrap();
//         let server_process_result = app::read_next_publisher_data_message(&server_socket, &mut reusable_buffer, 17).unwrap();

//         // Assert no new data message
//         assert_eq!(true, server_process_result.is_none());

//         // Assert expected reply sent to client
//         let reply = receive_string(&client_socket);
//         assert_eq!("SUBSCRIBER_COUNT 17", reply.as_str());
//     }

//     #[test]
//     fn data_expect_message_ended_up_in_channel_test()
//     {
//         // Init
//         let (client_socket, server_socket, mut reusable_buffer) = (create_socket_with_receive_timeout(), create_socket_with_receive_timeout(), [0; 10000]);
        
//         // Execute
//         client_socket.send_to("DATA This is the data provided \n in multiple \n\r lines".as_bytes(), server_socket.local_addr().unwrap()).unwrap();
//         let server_process_result = app::read_next_publisher_data_message(&server_socket, &mut reusable_buffer, 17).unwrap();

//         // Assert data message returned
//         assert_eq!(true, server_process_result.is_some());
//         assert_eq!("This is the data provided \n in multiple \n\r lines", server_process_result.unwrap().payload.as_str());
        
//         // Assert no reply sent on data messages
//         ensure_nothing_to_receive(&client_socket);
//     }
// }