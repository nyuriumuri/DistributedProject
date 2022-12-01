
mod receiver;
use std::{env, time::Duration, thread, fs, io::Write};
use receiver::RequestReceiver;
fn main() {
    let args: Vec<String> = env::args().collect();  // get server address

    let receive_addr = &args[1];
    let index = {
        args[2].parse::<usize>().unwrap()
    }; 
    
    let mut receiver = RequestReceiver::new(receive_addr.to_string(), index);
    receiver.listen(); 
    receiver.log_stats();
    receiver.handle_requests();
    loop{
        thread::sleep(Duration::from_secs(10));
        receiver.election(); 
        let mut file = fs::File::create({
            let fname = receive_addr.to_string().clone();
            format!("{}-times_elected.txt", fname.replace(":", "-"))
        }).unwrap();
        writeln!(file, "{}", receiver.times_elected).unwrap();
    }
    // t1.join().unwrap();
    // t2.join().unwrap();
    // t3.join().unwrap();
    
}



