// use  std::sync::mpsc::Sender;
use std::net::{UdpSocket};
use std::str;

pub struct RequestReceiver{ // struct to hold receiver info
    addr: String, // local address
    socket: UdpSocket, // socket to receive requests
}

impl RequestReceiver{ // impl RequestReceiver
    pub fn new(addr: String) -> RequestReceiver { // constructor
        RequestReceiver{ // return RequestReceiver
            addr: addr.clone(), // set addr
            socket: UdpSocket::bind(addr).expect("couldn't bind sender to address"), // bind socket to addr
        } // return RequestReceiver
    } // end constructor

    pub fn listen(&self){ // listen function 
        println!("Listening on port {}", self.addr); // print listening message 
        loop{ 
                let mut buf = [0; 1000]; // create buffer to hold message 
                let (_, src_addr) = self.socket.recv_from(&mut buf).expect("Didn't receive data"); // receive message
                let buf_str = str::from_utf8(&buf[1..]).unwrap(); // convert message to string
                println!("{}", buf_str); // print message
                if buf[0] == 0  // first byte is 1 if sent message is acknowlegement, 0 if a request
                {
                    let mut reply_addr = src_addr; // clone src_addr to reply_addr
                    reply_addr.set_port(reply_addr.port()+1); // set reply_addr port to src_addr port + 1
                    let mut message = String::from(format!("Message Recieved By {}", self.addr)); // create message
                    let mut message = message.into_bytes(); // convert message to bytes
                    message.insert(0,1); // insert 1 as first byte

                    self.socket.send_to(&message, reply_addr).expect("Failed to send acknowledgement"); // send acknowledgement
                } // end if
        } // end loop
    } // end listen function

}