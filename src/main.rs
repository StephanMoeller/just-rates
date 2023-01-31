mod app;
use std::{
    net::UdpSocket,
};

fn main() -> std::io::Result<()> {
    // Hardcoded input parameters
    let publisher_udp_port: i32 = 11311;
    let websocket_tcp_port: u16 = 8081;
    _start_client_spammer(publisher_udp_port);

    // SETUP udp listening socket and websocket server
    let udp_addr = format!("127.0.0.1:{publisher_udp_port}");
    let udp_socket = std::net::UdpSocket::bind(&String::from(&udp_addr))
        .expect(format!("Error binding udp on local address {udp_addr}").as_str());
    let websocket_event_hub = simple_websockets::launch(websocket_tcp_port)
        .expect(format!("failed to listen on websocket port {websocket_tcp_port}").as_str());

    // Actual application
    app::run(udp_socket, websocket_event_hub).unwrap();
    return Ok(());
}

fn _start_client_spammer(publisher_udp_port: i32)
{
    // POC udp message spammer
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    let server_addr = format!("127.0.0.1:{publisher_udp_port}");
    std::thread::spawn(move || {
        let mut counter: i32 = 0;

        loop {
            counter += 1;
            client.send_to(format!("DATA This is data message number {counter}").as_bytes(), &server_addr).unwrap();
            
        }
    });
}
