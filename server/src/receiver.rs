// use  std::sync::mpsc::Sender;
use std::net::{UdpSocket};
use std::str;
use rand::prelude::{random};

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
                let buf_str = str::from_utf8(&buf[..]).unwrap();
                println!("{}", buf_str);
                println!("Got a message from {}", src_addr);
                if buf[0] == 0  // first byte is 1 if sent message is acknowlegement, 0 if a request
                {
                    let reply_addr = src_addr;
                    let message: u16 = random();
                    println!("{}",message);
                    let message = message.to_be_bytes();
                    // reply_addr.set_port(reply_addr.port()+1);
                    // let mut message = String::from(format!("Message Recieved By {}", self.addr));
                    // let mut message = message.into_bytes();
                    // message.insert(0,1);
                    // println!("Got a message");
                    self.socket.send_to(&message, reply_addr).expect("Failed to send acknowledgement");
                }
        }
    }

}