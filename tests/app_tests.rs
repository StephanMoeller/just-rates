mod test_utils;

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use rust_just_rates::app;
    use url::Url;
    use tungstenite::{connect, Message};

    #[test]
    fn run_no_subscribers_test()
    {
        let websocket_port: u16 = 9000;

        let udp_socket = std::net::UdpSocket::bind(&String::from("127.0.0.1:0")).unwrap();
        let websocket_event_hub = simple_websockets::launch(websocket_port).unwrap();
        
        let server_udp_endpoint = udp_socket.local_addr().unwrap();

        std::thread::spawn(||{
            app::run(udp_socket, websocket_event_hub).unwrap();
        });

        // Get subscriber count
        let publisher_client = create_socket_with_receive_timeout();
        publisher_client.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client);
        assert_eq!("SUBSCRIBER_COUNT 0", reply);

        // Send some data and ensure no error returned
        publisher_client.send_to("DATA Some data content here".as_bytes(), server_udp_endpoint).unwrap();
        ensure_nothing_to_receive(&publisher_client);
    }

    #[test]
    fn run_with_subscribers_test()
    {
        let websocket_port: u16 = 9001;

        let udp_socket = std::net::UdpSocket::bind(&String::from("127.0.0.1:0")).unwrap();
        let websocket_event_hub = simple_websockets::launch(websocket_port).unwrap();

        let server_udp_endpoint = udp_socket.local_addr().unwrap();

        std::thread::spawn(||{
            app::run(udp_socket, websocket_event_hub).unwrap();
        });

        
        let publisher_client_1 = create_socket_with_receive_timeout();
        let publisher_client_2 = create_socket_with_receive_timeout();

        // Ensure no subscribers
        publisher_client_1.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), &server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client_1);
        assert_eq!("SUBSCRIBER_COUNT 0", reply);

        // Connect some subscribers
        let server_websocket_endpoint = format!("ws://127.0.0.1:{websocket_port}");
        let (mut websocket_client_A, response_A) = tungstenite::connect(Url::parse(server_websocket_endpoint.as_str()).unwrap()).expect("Can't connect");
        let (mut websocket_client_B, response_B) = tungstenite::connect(Url::parse(server_websocket_endpoint.as_str()).unwrap()).expect("Can't connect");

        // Now ensure 2 subscribers
        publisher_client_1.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), &server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client_1);
        assert_eq!("SUBSCRIBER_COUNT 2", reply);

        ensure_nothing_to_receive(&publisher_client_1);
        ensure_nothing_to_receive(&publisher_client_2);

        websocket_client_A.close(None).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(500));

        // Ensure one less subscriber now
        publisher_client_1.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), &server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client_1);
        assert_eq!("SUBSCRIBER_COUNT 1", reply);
     
        websocket_client_B.close(None).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(500));

        publisher_client_1.send_to("GET_SUBSCRIBER_COUNT".as_bytes(), &server_udp_endpoint).unwrap();
        let reply = receive_string(&publisher_client_1);
        assert_eq!("SUBSCRIBER_COUNT 0", reply);
    }
}