use rust_just_rates::app;


#[cfg(test)]
mod tests {
    use std::net::UdpSocket;

    use super::*;

    #[test]
    fn send_receive_test() {
        const SERVER_ADDR: &str = "127.0.0.1:34254";
        // TODO: Let OS choose a port
        let app_thread = std::thread::spawn(|| {            
            app::run(&String::from(SERVER_ADDR)).unwrap();
        });
        
        let client = UdpSocket::bind(String::from("127.0.0.1:8888")).unwrap();
        let byte_count_to_send: usize = 14;
        let mut buffer: [u8; 100] = [0; 100];
        
        let mut counter : u8 = 0;
        for i in 0..byte_count_to_send
        {
            counter = counter + 1;
            buffer[i] = counter;
        }
        
        println!("Buffer {:?}", &mut buffer[0..byte_count_to_send]);
        client.send_to(&mut buffer[0..byte_count_to_send], &SERVER_ADDR).unwrap();
        println!("Waiting to receive");
        let (amt, src) = client.recv_from(&mut buffer).unwrap();
        
        println!("Received {:?}", &mut buffer[0..amt]);
        assert_eq!(byte_count_to_send, amt);
        assert_eq!(src.to_string(), "127.0.0.1:34254");
        assert_eq!(false, app_thread.is_finished());
    }
}