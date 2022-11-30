// use  std::sync::mpsc::Receiver;
use std::net::UdpSocket; // use UdpSocket
use std::io; // use io

pub struct RequestSender{ // struct to hold sender info
    addr: String, // local address
    socket: UdpSocket, // socket to send requests
    recepient_addr: String, // address to send requests to
}

impl RequestSender{ // impl RequestSender
    pub fn new(addr: String, recepient_addr: String) -> RequestSender { // constructor
        RequestSender{ // return RequestSender
            addr: addr.clone(), // set addr
            recepient_addr: recepient_addr.clone(), // set recepient_addr
            socket: UdpSocket::bind(addr).expect("couldn't bind reciever to address") // bind socket to addr
        } // return RequestSender
    } // end constructor


    pub fn run(&self){ // run function
        loop { // loop
            let mut message = String::new(); // create string to hold message
            io::stdin().read_line(&mut message).expect("failed to read line"); // read message from stdin
            io::stdin().read_line(&mut message).unwrap(); // read message from stdin
            let mut message = String::from(message.trim()).into_bytes(); // convert message to bytes
            message.insert(0,0); // insert 0 at beginning of message
            self.socket.send_to(&message, self.recepient_addr.clone()).unwrap(); // send message to recepient_addr
        } // end loop
    } // end run function
}