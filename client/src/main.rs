

// to run as server:
// cargo run <server_address> <client_address> 0 
// cargo run 127.0.0.1:7878 127.0.0.1:5656 0

// to run as client
// cargo run <client_address> <server_address> 1
// cargo run 127.0.0.1:5656 127.0.0.1:7878 1


mod receiver;
mod sender;
// use std::thread;
use std::env;
// use receiver::RequestReceiver;
use sender::RequestSender;
fn main() {
    let args: Vec<String> = env::args().collect();  // get local and remote addresses
    let send_addr = &args[1];
     
    // let receiver = RequestReceiver::new(receive_addr.to_string());
    let mut sender = RequestSender::new(send_addr.to_string());
    let t = sender.init();
    t.join().unwrap();
    
}



