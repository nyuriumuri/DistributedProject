
mod receiver;
// mod sender;
use std::env;
use receiver::RequestReceiver;
// use sender::RequestSender;
fn main() {
    let args: Vec<String> = env::args().collect();  // get local and remote addresses

    let receive_addr = &args[1];

    
    let receiver = RequestReceiver::new(receive_addr.to_string());
    // let sender = RequestSender::new(send_addr.to_string(), remote.to_string());
    let t1 = receiver.listen();
    receiver.handle_requests();

    t1.join().unwrap();
    // t2.join().unwrap();
    
}



