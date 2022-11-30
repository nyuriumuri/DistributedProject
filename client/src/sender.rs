
// use  std::sync::mpsc::Receiver;
use serde::{Deserialize, Serialize}; // use serde
use std::net::UdpSocket; // use UdpSocket
use std::sync::{Arc, Mutex}; // use Arc and Mutex
use serde_json; // use serde_json
use std::thread; // use thread
use std::time::{self, Instant, Duration}; // use time
use std::fs; // use fs

#[derive(Deserialize, Serialize, Debug)] // derive Deserialize, Serialize, and Debug
struct ServerStat { // struct to hold server stats
    name: String, // name of server
    num_sent: u32, // number of messages sent
    num_ackd : u32 // number of messages ackd
  }
  
#[derive(Deserialize, Serialize, Debug)] // derive Deserialize, Serialize, and Debug
pub struct ClientStats { // struct to hold client stats
    name: String, // name of client
    total_sent: u128, // total number of messages sent
    total_successful: u128, // total number of messages ackd
    avg_response_time: u128, // average response time
    server_stats: Vec<ServerStat>, // vector of server stats
  }

pub struct RequestSender{ // struct to hold sender info 
  //  addr: String,
    name: String, // name of client
    rec_socket: Arc<Mutex<UdpSocket>>, // socket to receive requests
    send_socket: Arc<Mutex<UdpSocket>>, // socket to send requests
    servers: Vec<String>, // vector of server addresses
    min_server: Arc<Mutex<u16>>, // index of server with lowest number of messages sent
    min_server_load: Arc<Mutex<u16>>, // number of messages sent by server with lowest number of messages sent
    messages_per_server: [(u32, u32); 3],   //0: total messages sent  | 1: messages successfully received 
    avg_request_time : (u128, u128),  // 0: total successful messages,  1: average time per successful request
    total_sent: u128 // total number of messages sent

    // recepient_addr: String, 
}

const SERVERS_JSON : &str = "servers.json"; // constant for servers.json

impl RequestSender{ // impl RequestSender
    pub fn new(rec_addr: String, send_addr: String, name: String) -> RequestSender { // constructor
        let servers_string = fs::read_to_string(SERVERS_JSON).expect("Could not read server json file");   // read servers.json 
        let servers = serde_json::from_str(&servers_string).expect("Could not deserialize json"); // deserialize json
        let send_socket = UdpSocket::bind(send_addr).expect("couldn't bind sender to address"); // bind socket to send_addr
        send_socket.set_read_timeout(Some(time::Duration::from_millis(900))).unwrap(); // set read timeout to 900ms
        RequestSender{ // return RequestSender
            name: name,  // set name
            servers: servers, // set servers
            min_server: Arc::new(Mutex::new(0)), // set min_server to 0
            min_server_load: Arc::new(Mutex::new(u16::MAX)), // set min_server_load to u16::MAX
            rec_socket: Arc::new(Mutex::new(UdpSocket::bind(rec_addr).expect("couldn't bind sender to address"))), // bind socket to rec_addr
            send_socket: Arc::new(Mutex::new(send_socket)), // set send_socket
            messages_per_server : [(0,0); 3], // set messages_per_server to [(0,0); 3]
            avg_request_time: (0,0), // set avg_request_time to (0,0)
            total_sent: 0 // set total_sent to 0
        }
    }


    pub fn init(&self) -> thread::JoinHandle<()>{ // init function
        let s_socket  = Arc::clone(&self.rec_socket); // clone rec_socket
        let s_min_server_load = Arc::clone(&self.min_server_load); // clone min_server_load
        let s_min_server = Arc::clone(&self.min_server); // clone min_server
        let servers = self.servers.clone(); // clone servers
        // socket = self.socket.clone();
        return thread::spawn(move || {  // spawn thread
            let buf : [u8;2]= [2, 2]; // set buf to [2,2]
            let socket_lock = s_socket.lock().unwrap(); // lock socket
            socket_lock.set_read_timeout(Some(time::Duration::from_secs(1))).unwrap(); // set read timeout to 1s
            drop(socket_lock); // drop socket_lock
            loop
            {   
                // self.min_server_load = u16::MAX;
                
                for (i, server) in servers.iter().enumerate() // iterate through servers 
                {
                    let socket_lock = s_socket.lock().unwrap(); // lock socket
                    socket_lock.send_to(&buf, server).expect("Failed to send pulse"); // send buf to server
                    let mut recv_buf = [0; 2];  // set recv_buf to [0;2]
                    let recv_res = socket_lock.recv_from(&mut recv_buf); // set recv_res to recv_from
                    drop(socket_lock);  // drop socket_lock
                    match recv_res{ // match recv_res
                        Ok((_,_)) => { // if Ok
                            let mut min_server_load = s_min_server_load.lock().unwrap(); // lock min_server_load
                            let mut min_server = s_min_server.lock().unwrap(); // lock min_server
                            let val = u16::from_be_bytes(recv_buf); // set val to u16::from_be_bytes(recv_buf)
                           // print!("{}: {} | {:?}\n", i, val, recv_buf);
                            if val < *min_server_load{ // if val < min_server_load
                                *min_server_load = val; // set min_server_load to val
                                *min_server = i as u16; // set min_server to i as u16
                            }
                        }, // end if Ok

                        Err(_) => (
                            // println!("{}: {}", i, "No response")
                        ) // if Err, do nothing
                    }; // end match

                }
                let mut min_server_load = s_min_server_load.lock().unwrap(); // lock min_server_load
                let min_server = s_min_server.lock().unwrap(); // lock min_server
               // println!("Load: {} \nServer: {}",*min_server_load, *min_server);
                *min_server_load = u16::MAX; // set min_server_load to u16::MAX

                drop(min_server_load); // drop min_server_load
                drop(min_server); // drop min_server
                thread::sleep(time::Duration::from_secs(5)); // sleep for 5s
            } // end loop
         }); // end return
    }   // end init

    pub fn send(&mut self, message: String){ // send function
        let socket = self.send_socket.lock().unwrap(); // lock send_socket
        let message_buf = message.clone().into_bytes(); // set message_buf to message.clone().into_bytes()
        let min_server = self.min_server.lock().unwrap(); // lock min_server
        let min_server = min_server.clone();  // set min_server to min_server.clone()
        socket.send_to(&message_buf, &self.servers[min_server as usize]).unwrap(); // send message_buf to servers[min_server as usize]
        self.messages_per_server[usize::from(min_server)].0+=1;  // increment messages_per_server[usize::from(min_server)].0
        self.total_sent+=1;  // increment total_sent
        
        let mut buf: [u8; 100] = [0; 100];   // set buf to [0;100]
        let time = Instant::now();  // set time to Instant::now()
        while time.elapsed() < Duration::from_millis(900){ // while time.elapsed() < Duration::from_millis(900)
            match socket.recv_from(&mut buf){ // match socket.recv_from(&mut buf)
                Ok(_) =>  { // if Ok
                    //println!("{}", String::from_utf8(buf.to_vec()).unwrap());
                    let reply_str = String::from_utf8(buf.to_vec()).unwrap(); // set reply_str to String::from_utf8(buf.to_vec()).unwrap() 
                    let reply_str = { // set reply_str to
                        let r = reply_str.trim_matches(char::from(0)); // set r to reply_str.trim_matches(char::from(0))
                        String::from(r) // return String::from(r)
                    };
                    if reply_str.trim().eq(message.trim())  // if reply matches the sent request 
                    { 
                        // println!("SUCCESS Expected |{}| GOT |{}|", message.trim(), reply_str.trim());
                        self.messages_per_server[usize::from(min_server)].1+=1;  // increment messages_per_server[usize::from(min_server)].1
                        
                        self.avg_request_time.1 = (time.elapsed().as_micros() + self.avg_request_time.0*self.avg_request_time.1) / (self.avg_request_time.0 + 1);  
                        // println!("{}", time.elapsed().as_micros()); 
                        self.avg_request_time.0 +=1; // increment avg_request_time.0
                        break;  // break
                    }else{
                        // println!("ERR Expected |{:?}| GOT |{:?}|", message.trim().as_bytes(), reply_str.trim().as_bytes());
                    }
                },
                Err(_) => {
                // println!("Missed Timing");
                },  
            }
    }
    }

    pub fn get_stats(&self) -> ClientStats{ // get_stats function
        let mut client_stats = ClientStats{name: self.name.clone(), server_stats: vec![], total_sent: self.total_sent.clone(),  total_successful: self.avg_request_time.0.clone(), avg_response_time: self.avg_request_time.1};
        let mut stats = vec![]; // set stats to vec![]
        stats.push(format!("\n{} :", self.name).to_string()); // push format!("\n{} :", self.name).to_string() to stats
        stats.push(String::from("Server\tSent\tAcknowledged")); // push "Server\tSent\tAcknowledged" to stats
        for (i, server_stats) in self.messages_per_server.into_iter().enumerate()  { 
            stats.push(format!("{}\t{}\t{}", i, server_stats.0, server_stats.1)); // push format!("{}\t{}\t{}", i, server_stats.0, server_stats.1) to stats
            client_stats.server_stats.push(ServerStat{name: format!("{}",i), num_sent: server_stats.0, num_ackd: server_stats.1});  // push ServerStat{name: format!("{}",i), num_sent: server_stats.0, num_ackd: server_stats.1} to client_stats.server_stats
        }
        print!("{}", stats.join("\n")); // print stats.join("\n")
        client_stats // return client_stats
    } // end get_stats
} // end impl Client