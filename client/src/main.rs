

// to run as server:
// cargo run <server_address> <client_address> 0 
// cargo run 127.0.0.1:7878 127.0.0.1:5656 0

// to run as client
// cargo run <client_address> <server_address> 1
// cargo run 127.0.0.1:5656 127.0.0.1:7878 1



mod sender;
// use std::thread;
use std::{env, fs};
// use std::io;
use std::net::{SocketAddrV4};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
// use receiver::RequestReceiver;
use sender::{RequestSender, ClientStats};



fn main() {
    let args: Vec<String> = env::args().collect();  // get local and remote addresses
    let send_addr = &args[1];
    let mut threads = vec![];
    let stats_arc = {
      let s: Vec<ClientStats> = vec![];
      Arc::new(Mutex::new(s)) 
    };

    // let receiver = RequestReceiver::new(receive_addr.to_string());
    
    for i in 0..300{
        let mut send_addr = SocketAddrV4::from_str(send_addr).unwrap();
        send_addr.set_port(send_addr.port()+i);
        let mut rec_addr = send_addr.clone();
        rec_addr.set_port(rec_addr.port()+1000);
        let stats_arc_ = stats_arc.clone();
        let loop_fun = move || {
          let mut sender = RequestSender::new(send_addr.to_string(), rec_addr.to_string(), format!("Client {}", i));
          sender.init();
            for j in 0..100{

            
                // let mut message = random();
                // io::stdin().read_line(&mut message).unwrap();
                // let message = String::from(message.trim());
                sender.send(String::from(format!("Hello from {} [{}]", send_addr.to_string(), j)));
                thread::sleep(Duration::from_secs(1));
                
            }
          let mut stats_vec = stats_arc_.lock().unwrap();
          stats_vec.push(sender.get_stats()); 
        };
        threads.push(thread::spawn(move || loop_fun()));

    }
 
  
    for t in threads{
      t.join().unwrap();
    }
    let stats_vec = stats_arc.lock().unwrap();
    let stats_string = serde_json::to_string_pretty(&*stats_vec).unwrap(); 
    fs::write("stats.json", stats_string).unwrap(); 
   // t.join().unwrap();
    
    
}



