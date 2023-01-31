mod app;
use std::{
    net::UdpSocket,
};

fn main() -> std::io::Result<()> {
    // Hardcoded input parameters
    let publisher_udp_port: i32 = 11311;
    let websocket_tcp_port: i32 = 8081;

    // Actual application
    app::run(publisher_udp_port, websocket_tcp_port).unwrap();
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
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
    });
}
