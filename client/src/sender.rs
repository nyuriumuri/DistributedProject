
// use  std::sync::mpsc::Receiver;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use serde_json;
use std::thread;
use std::time::{self, Instant, Duration};
use std::fs;
#[derive(Deserialize, Serialize, Debug)]
struct ServerStat {
    name: String, 
    num_sent: u32,
    num_ackd : u32 
  }
  
#[derive(Deserialize, Serialize, Debug)]
pub struct ClientStats {
    name: String, 
    total_sent: u128, 
    total_successful: u128, 
    avg_response_time: u128,
    server_stats: Vec<ServerStat>,

  }
pub struct RequestSender{
  //  addr: String,
    name: String, 
    rec_socket: Arc<Mutex<UdpSocket>>,
    send_socket: Arc<Mutex<UdpSocket>>,
    servers: Vec<String>,
    min_server: Arc<Mutex<u16>>,
    min_server_load: Arc<Mutex<u16>>,
    messages_per_server: [(u32, u32); 3],   //0: total messages sent  | 1: messages successfully received 
    avg_request_time : (u128, u128),  // 0: total successful messages,  1: average time per successful request
    total_sent: u128

    // recepient_addr: String, 
}

const SERVERS_JSON : &str = "servers.json";
impl RequestSender{
    pub fn new(rec_addr: String, send_addr: String, name: String) -> RequestSender {
        let servers_string = fs::read_to_string(SERVERS_JSON).expect("Could not read server json file");   
        let servers = serde_json::from_str(&servers_string).expect("Could not deserialize json");  
        let send_socket = UdpSocket::bind(send_addr).expect("couldn't bind sender to address");
        send_socket.set_read_timeout(Some(time::Duration::from_millis(900))).unwrap();
        RequestSender{
            name: name, 
            servers: servers,
            min_server: Arc::new(Mutex::new(0)),
            min_server_load: Arc::new(Mutex::new(u16::MAX)),
            rec_socket: Arc::new(Mutex::new(UdpSocket::bind(rec_addr).expect("couldn't bind sender to address"))),
            send_socket: Arc::new(Mutex::new(send_socket)),
            messages_per_server : [(0,0); 3],
            avg_request_time: (0,0),
            total_sent: 0

        }
    }


    pub fn init(&self) -> thread::JoinHandle<()>{
        let s_socket  = Arc::clone(&self.rec_socket);
        let s_min_server_load = Arc::clone(&self.min_server_load);
        let s_min_server = Arc::clone(&self.min_server);
        let servers = self.servers.clone();
        // socket = self.socket.clone();
        return thread::spawn(move || { 
            let buf : [u8;2]= [2, 2];
            let socket_lock = s_socket.lock().unwrap();
            socket_lock.set_read_timeout(Some(time::Duration::from_secs(1))).unwrap();
            drop(socket_lock);
            loop
            {   
                // self.min_server_load = u16::MAX;
                
                for (i, server) in servers.iter().enumerate()
                {
                    let socket_lock = s_socket.lock().unwrap();
                    socket_lock.send_to(&buf, server).expect("Failed to send pulse");
                    let mut recv_buf = [0; 2]; 
                    let recv_res = socket_lock.recv_from(&mut recv_buf);
                    drop(socket_lock); 
                    match recv_res{
                        Ok((_,_)) => {
                            let mut min_server_load = s_min_server_load.lock().unwrap();
                            let mut min_server = s_min_server.lock().unwrap();
                            let val = u16::from_be_bytes(recv_buf);
                           // print!("{}: {} | {:?}\n", i, val, recv_buf);
                            if val < *min_server_load{
                                *min_server_load = val;
                                *min_server = i as u16;
                            }
                        },

                        Err(_) => ()
                    };

                }
                let mut min_server_load = s_min_server_load.lock().unwrap();
                let min_server = s_min_server.lock().unwrap();
               // println!("Load: {} \nServer: {}",*min_server_load, *min_server);
                *min_server_load = u16::MAX;

                drop(min_server_load);
                drop(min_server);
                thread::sleep(time::Duration::from_secs(5));
            }
         });
        
    }   

    pub fn send(&mut self, message: String){
        let socket = self.send_socket.lock().unwrap();
        let message_buf = message.clone().into_bytes();
        let min_server = self.min_server.lock().unwrap();
        let min_server = min_server.clone(); 
        socket.send_to(&message_buf, &self.servers[min_server as usize]).unwrap();
        self.messages_per_server[usize::from(min_server)].0+=1; 
        self.total_sent+=1; 
        
        let mut buf: [u8; 100] = [0; 100];  
        let time = Instant::now(); 
        while time.elapsed() < Duration::from_millis(900){
            match socket.recv_from(&mut buf){
                Ok(_) =>  {
                    //println!("{}", String::from_utf8(buf.to_vec()).unwrap());
                    let reply_str = String::from_utf8(buf.to_vec()).unwrap();
                    let reply_str = {
                        let r = reply_str.trim_matches(char::from(0));
                        String::from(r)
                    };
                    if reply_str.trim().eq(message.trim())  // if reply matches the sent request 
                    { 
                        // println!("SUCCESS Expected |{}| GOT |{}|", message.trim(), reply_str.trim());
                        self.messages_per_server[usize::from(min_server)].1+=1; 
                        
                        self.avg_request_time.1 = (time.elapsed().as_micros() + self.avg_request_time.0*self.avg_request_time.1) / (self.avg_request_time.0 + 1); 
                        // println!("{}", time.elapsed().as_micros()); 
                        self.avg_request_time.0 +=1;
                        break; 
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

    pub fn get_stats(&self) -> ClientStats{
        let mut client_stats = ClientStats{name: self.name.clone(), server_stats: vec![], total_sent: self.total_sent.clone(), total_successful: self.avg_request_time.0.clone(), avg_response_time: self.avg_request_time.1}; 
        let mut stats = vec![]; 
        stats.push(format!("\n{} :", self.name).to_string());
        stats.push(String::from("Server\tSent\tAcknowledged"));
        for (i, server_stats) in self.messages_per_server.into_iter().enumerate()  {
            stats.push(format!("{}\t{}\t{}", i, server_stats.0, server_stats.1));
            client_stats.server_stats.push(ServerStat{name: format!("{}",i), num_sent: server_stats.0, num_ackd: server_stats.1});
        }
        print!("{}", stats.join("\n"));    
        client_stats
        
    }
}