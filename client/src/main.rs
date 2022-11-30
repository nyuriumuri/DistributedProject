
mod sender;
// use std::thread;
use std::{env, fs};
// use std::io;
use std::net::{SocketAddrV4};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
// use std::time::Duration;
// use receiver::RequestReceiver;
use sender::{RequestSender, ClientStats};


fn main() { 
    let args: Vec<String> = env::args().collect();  // get server address
    let send_addr = &args[1]; // local address

    let mut threads = vec![]; // vector to hold all threads

    let stats_arc = { 
      let s: Vec<ClientStats> = vec![];
      Arc::new(Mutex::new(s)) 
    }; // Arc to hold all client stats

    
    for i in 0..500{
        let mut send_addr = SocketAddrV4::from_str(send_addr).unwrap(); // convert to SocketAddrV4

        send_addr.set_port(send_addr.port()+i); // set port to local port + i

        let mut rec_addr = send_addr.clone(); // clone send_addr to rec_addr

        rec_addr.set_port(rec_addr.port()+1000); // set rec_addr port to send_addr port + 1000

        let stats_arc_ = stats_arc.clone(); // clone stats_arc

        let loop_fun = move || { // closure to be run in thread 
          let mut sender = RequestSender::new(send_addr.to_string(), rec_addr.to_string(), format!("Client {}", i)); // create sender
          sender.init(); // initialize sender
            for j in 0..1000{

                sender.send(String::from(format!("Hello from {} [{}]", send_addr.to_string(), j))); // send message
                // thread::sleep(Duration::from_secs(1));

                
            }
          let mut stats_vec = stats_arc_.lock().unwrap(); // lock stats_arc
          stats_vec.push(sender.get_stats()); // push sender stats to stats_vec
        };
        threads.push(thread::spawn(move || loop_fun())); // spawn thread
    }
 
    for t in threads{
      t.join().unwrap(); // join all threads
    }

    let stats_vec = stats_arc.lock().unwrap(); // lock stats_arc
    let stats_string = serde_json::to_string_pretty(&*stats_vec).unwrap(); // convert stats_vec to json string
    fs::write("stats.json", stats_string).unwrap(); // write stats_string to stats.json
}



