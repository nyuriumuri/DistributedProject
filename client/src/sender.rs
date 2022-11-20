// use  std::sync::mpsc::Receiver;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::{fs};
use serde_json;
use std::thread;
use std::time;

pub struct RequestSender{
    addr: String,
    rec_socket: Arc<Mutex<UdpSocket>>,
    send_socket: Arc<Mutex<UdpSocket>>,
    servers: Vec<String>,
    min_server: Arc<Mutex<u16>>,
    min_server_load: Arc<Mutex<u16>>,

    // recepient_addr: String, 
}

const SERVERS_JSON : &str = "servers.json";
impl RequestSender{
    pub fn new(rec_addr: String, send_addr: String) -> RequestSender {
        let servers_string = fs::read_to_string(SERVERS_JSON).expect("Could not read server json file");   
        let servers = serde_json::from_str(&servers_string).expect("Could not deserialize json");  
        RequestSender{
            addr: addr.clone(),
            servers: servers,
            min_server: Arc::new(Mutex::new(0)),
            min_server_load: Arc::new(Mutex::new(u16::MAX)),
            rec_socket: Arc::new(Mutex::new(UdpSocket::bind(rec_addr).expect("couldn't bind sender to address"))),
            send_socket: Arc::new(Mutex::new(UdpSocket::bind(send_addr).expect("couldn't bind sender to address")))

        }
    }


    pub fn init(&self) -> thread::JoinHandle<()>{
        let s_socket  = Arc::clone(&self.recv_socket);
        let s_min_server_load = Arc::clone(&self.min_server_load);
        let s_min_server = Arc::clone(&self.min_server);
        let servers = self.servers.clone();
        // socket = self.socket.clone();
        return thread::spawn(move || { 
            let buf : [u8;2]= [0, 0];
            let socket_lock = s_socket.lock().unwrap();
            socket_lock.set_read_timeout(Some(time::Duration::from_secs(4))).unwrap();
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
                            if val < *min_server_load{
                                *min_server_load = val;
                                *min_server = i as u16;
                            }
                        },

                        Err(_) => println!("Listen Failed")
                    };

                }
                let mut min_server_load = s_min_server_load.lock().unwrap();
                let mut min_server = s_min_server.lock().unwrap();
                println!("Load: {} \nServer: {}",*min_server_load, *min_server);
                *min_server_load = u16::MAX;

                drop(min_server_load);
                drop(min_server);
                thread::sleep(time::Duration::from_secs(5));
            }
         });
        
    }   

    pub fn send(&self, message: String){
        let socket = self.send_socket.lock().unwrap();
        let mut message_buf = message.into_bytes();
        // let mut message = String::new();
        // io::stdin().read_line(&mut message).unwrap();
        // let mut message = String::from(message.trim()).into_bytes();
        // message_buf.insert(1,0);
        let min_server = self.min_server.lock().unwrap();
        socket.send_to(&message_buf, &self.servers[*min_server as usize]).unwrap();
    }
}