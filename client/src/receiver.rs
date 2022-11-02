// use  std::sync::mpsc::Sender;
use std::net::{UdpSocket};
use std::str;

pub struct RequestReceiver{
    addr: String,
    socket: UdpSocket
}


impl RequestReceiver{
    pub fn new(addr: String) -> RequestReceiver {
        RequestReceiver{
            addr: addr.clone(),
            socket: UdpSocket::bind(addr).expect("couldn't bind sender to address")
        }
    }

    pub fn listen(&self){
        println!("Listening on port {}", self.addr);
        loop{
                let mut buf = [0; 1000];    
                let (_, src_addr) = self.socket.recv_from(&mut buf).expect("Didn't receive data");
                let buf_str = str::from_utf8(&buf[1..]).unwrap();
                println!("{}", buf_str);
                if buf[0] == 0  // first byte is 1 if sent message is acknowlegement, 0 if a request
                {
                    let mut reply_addr = src_addr;
                    // reply_addr.set_port(reply_addr.port()+1);
                    let mut message = String::from(format!("Message Recieved By {}", self.addr));
                    let mut message = message.into_bytes();
                    message.insert(0,1);

                    self.socket.send_to(&message, reply_addr).expect("Failed to send acknowledgement");
                }
        }
    }

}