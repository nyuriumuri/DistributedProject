
mod receiver;
use std::env;
use receiver::RequestReceiver;
fn main() {
    let args: Vec<String> = env::args().collect();  // get server address

    let receive_addr = &args[1];

    
    let receiver = RequestReceiver::new(receive_addr.to_string());
    let t1 = receiver.listen(); 
    let t2 = receiver.log_stats();
    receiver.handle_requests();

    t1.join().unwrap();
    t2.join().unwrap();
    
}



