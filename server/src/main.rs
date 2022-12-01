
mod receiver;
use std::{env, time::Duration, thread};
use receiver::RequestReceiver;
fn main() {
    let args: Vec<String> = env::args().collect();  // get server address

    let receive_addr = &args[1];
    let index = {
        args[2].parse::<usize>().unwrap()
    }; 
    
    let mut receiver = RequestReceiver::new(receive_addr.to_string(), index);
    let t1 = receiver.listen(); 
    let t2 = receiver.log_stats();
    let t3 = receiver.handle_requests();
    loop{
        thread::sleep(Duration::from_secs(10));
        receiver.election(); 
    }
    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
    
}



