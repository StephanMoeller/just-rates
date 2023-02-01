mod test_utils;

#[cfg(test)]
mod tests {
    use std::net::{ToSocketAddrs, UdpSocket};

    use crate::test_utils::*;
    use rstest::rstest;
    use rust_just_rates::app;
    use simple_websockets::EventHub;
    use tungstenite::{connect, Message};
    use url::Url;

    fn init(websocket_port: u16) -> (UdpSocket, std::net::SocketAddr, EventHub, String) {
        let udp_socket = std::net::UdpSocket::bind(&String::from("127.0.0.1:0")).unwrap();
        let websocket_event_hub = simple_websockets::launch(websocket_port).unwrap();

        let server_udp_endpoint = udp_socket.local_addr().unwrap();

        return (udp_socket, server_udp_endpoint, websocket_event_hub, format!("ws://127.0.0.1:{websocket_port}"));
    }
    #[test]
    fn run_no_subscribers_test() {
        let (server_udp_socket, server_udp_endpoint, websocket_event_hub, _server_websocket_endpoint) = init(9000);
        
        std::thread::spawn(|| {
            app::run(server_udp_socket, websocket_event_hub).unwrap();
        });

        // Get subscriber count
        let publisher_client = create_socket_with_receive_timeout();
        publisher_client
            .send_to("GET_SUBSCRIBER_COUNT".as_bytes(), server_udp_endpoint)
            .unwrap();
        let reply = receive_string(&publisher_client);
        assert_eq!("SUBSCRIBER_COUNT 0", reply);

        // Send some data and ensure no error returned
        publisher_client
            .send_to(
                "DATA Some data content here".as_bytes(),
                server_udp_endpoint,
            )
            .unwrap();
        ensure_nothing_to_receive(&publisher_client);
    }

    #[test]
    fn run_with_subscribers_test() {
        let (server_udp_socket, server_udp_endpoint, websocket_event_hub, server_websocket_endpoint) = init(9001);

        std::thread::spawn(|| {
            app::run(server_udp_socket, websocket_event_hub).unwrap();
        });

        let publisher_client_1 = create_socket_with_receive_timeout();
        let publisher_client_2 = create_socket_with_receive_timeout();

        // Ensure no subscribers
        publisher_client_1.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), &server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client_1);
        assert_eq!("SUBSCRIBER_COUNT 0", reply);

        // Connect some subscribers
        
        let (mut websocket_client_a, _response_a) = tungstenite::connect(Url::parse(server_websocket_endpoint.as_str()).unwrap()).expect("Can't connect");
        let (mut websocket_client_b, _response_b) = tungstenite::connect(Url::parse(server_websocket_endpoint.as_str()).unwrap()).expect("Can't connect");

        // Now ensure 2 subscribers
        publisher_client_1.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), &server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client_1);
        assert_eq!("SUBSCRIBER_COUNT 2", reply);

        ensure_nothing_to_receive(&publisher_client_1);
        ensure_nothing_to_receive(&publisher_client_2);

        websocket_client_a.close(None).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(500));

        // Ensure one less subscriber now
        publisher_client_1.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), &server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client_1);
        assert_eq!("SUBSCRIBER_COUNT 1", reply);

        websocket_client_b.close(None).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(500));

        publisher_client_1.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), &server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client_1);
        assert_eq!("SUBSCRIBER_COUNT 0", reply);
    }

    #[rstest]
    #[case(9010, "INVALID MESSAGE HERE", "ERROR Unexpected protocol command: INVALID")]
    #[case(9011, "ERROR", "ERROR Client not allowed to send command ERROR")]
    #[case(9012, "ERROR With more data", "ERROR Client not allowed to send command ERROR")]
    #[case(9013, "SUBSCRIBER_COUNT","ERROR Client not allowed to send command SUBSCRIBER_COUNT")]
    #[case(9014, "SUBSCRIBER_COUNT With more data","ERROR Client not allowed to send command SUBSCRIBER_COUNT")]
    #[case(9015, "Get_SUBSCRIBER_COUNT","ERROR Unexpected protocol command: Get_SUBSCRIBER_COUNT")]
    #[case(9016, "DATA","ERROR Empty payload received after a DATA command which is not valid.")]
    #[case(9017, "Data", "ERROR Unexpected protocol command: Data")]
    #[case(9018, "DATA ","ERROR Empty payload received after a DATA command which is not valid.")]
    fn publisher_sends_invalid_message_expect_error_returned_test(
        #[case] websocket_server_port: u16,
        #[case] invalid_message_to_send: &str,
        #[case] expected_message_to_receive: &str,
    ) {
        let (udp_socket, server_udp_endpoint, websocket_event_hub, _server_websocket_endpoint) = init(websocket_server_port);
        let publisher_client = create_socket_with_receive_timeout();
        
        std::thread::spawn(|| {
            app::run(udp_socket, websocket_event_hub).unwrap();
        });

        // Execute
        publisher_client.send_to(invalid_message_to_send.as_bytes(),server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client);

        // Assert expected reply sent to client
        assert_eq!(expected_message_to_receive, reply);
    }

    #[test]
    fn publisher_sends_invalid_utf8_characters_expect_error_returned_test()
    {
        // Init
        let (udp_socket, server_udp_endpoint, websocket_event_hub, _server_websocket_endpoint) = init(9020);
        let publisher_client = create_socket_with_receive_timeout();
        
        std::thread::spawn(|| {
            app::run(udp_socket, websocket_event_hub).unwrap();
        });

        let mut invalid_utf8_bytes = "DATA Something more".as_bytes().to_owned();
        invalid_utf8_bytes[6] = 147; // Invalid utf8-character
        invalid_utf8_bytes[7] = 147; // Invalid utf8-character
        invalid_utf8_bytes[8] = 147; // Invalid utf8-character

        // Execute
        publisher_client.send_to(&invalid_utf8_bytes, server_udp_endpoint).unwrap();
        
        // Assert expected reply sent to client
        let reply = receive_string(&publisher_client);
        assert_eq!("ERROR Invalid utf8 bytes. Error details: invalid utf-8 sequence of 1 bytes from index 6", reply);
    }
}
