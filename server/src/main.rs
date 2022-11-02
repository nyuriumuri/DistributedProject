
mod receiver;
// mod sender;
use std::thread;
use std::env;
use receiver::RequestReceiver;
// use sender::RequestSender;
fn main() {
    let args: Vec<String> = env::args().collect();  // get local and remote addresses
    // let send_addr = &args[1];
    let receive_addr = &args[1];
    // let remote = &args[3];
    
    let receiver = RequestReceiver::new(receive_addr.to_string());
    // let sender = RequestSender::new(send_addr.to_string(), remote.to_string());
    let t1 = thread::spawn(
        move || {
            receiver.listen()
        }
    );
    // let t2 =thread::spawn(
    //     move || {
    //         sender.run()
    //     }
    // );

    t1.join().unwrap();
    // t2.join().unwrap();
    
}



