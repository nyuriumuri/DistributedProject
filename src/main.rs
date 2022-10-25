
// to run as server:
// cargo run <server_address> <client_address> 0 
// cargo run 127.0.0.1:7878 127.0.0.1:5656 0

// to run as client
// cargo run <client_address> <server_address> 1
// cargo run 127.0.0.1:5656 127.0.0.1:7878 1


use std::net::UdpSocket;
use std::{env, io};
fn main() {
    let args: Vec<String> = env::args().collect();  // get local and remote addresses
    let local = &args[1];
    let remote = &args[2];
    let is_sender =  &args[3] == "1" ;
    let socket  = UdpSocket::bind(local).expect("couldn't bind to address");
    println!("{local}");
    println!("{remote}");
    println!("{is_sender}");
    loop {
            if is_sender{                                                                                           // send a message if sender
                
           
            let mut message = String::new();
            io::stdin().read_line( &mut message).expect("Could not read input");
            let message = message.as_bytes();
            println!("Sending...");
            socket.send_to(message, remote.as_str()).expect("couldn't send data");
        }
            let mut buf = [0; 200];                                                                  // wait to recieve a message 
            let (number_of_bytes, src_addr) = 
                                        socket.recv_from(&mut buf)
                                        .expect("Didn't receive data");
            println!("received {number_of_bytes} from {src_addr}!");
            let buf = String::from_utf8(buf.to_vec()).expect("Could not convert to UTF-8");
            let buf = buf.as_str();
            println!("{buf}");
            if !is_sender {                                                                                 // send an acknowledgement if server
                let buf = String::from(format!("Acknowledged from Server {local}!"));
                let buf = buf.as_bytes();
                socket.send_to(buf, src_addr.to_string()).expect("couldn't send reply");
            }
    }
}



