use std::net::UdpSocket;

extern crate ctrlc;
mod app;

fn main() -> std::io::Result<()> {
    ctrlc::set_handler(|| {
        println!("Shutting down...");
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("Bye bye");
        std::process::exit(0x0100);
    }).expect("Error setting ctrlc handler");

    let local_socket = UdpSocket::bind(&String::from("127.0.0.1:34254")).unwrap();
    return app::run(local_socket);
}