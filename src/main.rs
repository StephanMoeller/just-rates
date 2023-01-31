
mod app;

fn main() -> std::io::Result<()> {
    let publisher_udp_port: i32 = 11311;
    let websocket_tcp_port: i32 = 8081;
    app::run(publisher_udp_port, websocket_tcp_port).unwrap();
    return Ok(());
}
