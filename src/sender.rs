// use  std::sync::mpsc::Receiver;
use std::net::UdpSocket;
use std::io;
pub struct RequestSender{
    addr: String,
    socket: UdpSocket,
    recepient_addr: String, 
}

impl RequestSender{
    pub fn new(addr: String, recepient_addr: String) -> RequestSender {
        RequestSender{
            addr: addr.clone(),
            recepient_addr: recepient_addr.clone(),
            socket: UdpSocket::bind(addr).expect("couldn't bind reciever to address")
        }
    }


    pub fn run(&self){
        loop
        {
            let mut message = String::new();
            io::stdin().read_line(&mut message).unwrap();
            let mut message = String::from(message.trim()).into_bytes();
            message.insert(0,0);
            self.socket.send_to(&message, self.recepient_addr.clone()).unwrap();
        }
    }   
}