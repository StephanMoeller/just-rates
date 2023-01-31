mod app;
use std::{net::{UdpSocket, TcpListener}, io::{Write, Read}};

fn main() -> std::io::Result<()> {

    // Hardcoded input parameters
    let publisher_udp_port: i32 = 11311;
    let websocket_tcp_port: i32 = 8081;

    // POC udp message spammer
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    let server_addr = format!("127.0.0.1:{publisher_udp_port}");
    std::thread::spawn(move || {
        let mut counter: i32 = 0;
        
        loop
        {
            counter += 1;
            client.send_to(format!("DATA This is data message number {counter}").as_bytes(), &server_addr).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(1000));
        }
    });

    // Serve static html file on seperate port
    let tcp_listener = TcpListener::bind("127.0.0.1:8082").unwrap();
    let file_content = std::fs::read_to_string("src/webclient/index.html").unwrap();
    
    let http_response = format!("HTTP/1.1 200 OK\r
Date: Mon, 27 Jul 2009 12:28:53 GMT\r
Server: Apache/2.2.14 (Win32)\r
Last-Modified: Wed, 22 Jul 2009 19:15:56 GMT\r
Content-Type: text/html\r
Connection: Closed\r
\r
{file_content}
    
");
    println!("{:?}", http_response);
    println!("{:?}", http_response.as_bytes());
    std::thread::spawn(move ||{
        let http_response_bytes = http_response.as_bytes();
        let mut buffer: [u8; 10000] = [0; 10000];

        loop{
            
            let (mut client_stream, _client_addr) = tcp_listener.accept().unwrap();
            
            let byte_count = client_stream.read(&mut buffer).unwrap();
            let request = std::str::from_utf8(&buffer[..byte_count]).unwrap();
            println!("::: REQUEST RECEIVED::: 
{}", request);
            println!("::: SENDING RESPONSE:::
{}", http_response);
            client_stream.write(http_response_bytes).unwrap();
            client_stream.flush().unwrap();
            
        }
    });

    // Actual application
    app::run(publisher_udp_port, websocket_tcp_port).unwrap();
    return Ok(());
}
