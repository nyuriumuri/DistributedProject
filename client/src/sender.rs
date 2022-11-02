// use  std::sync::mpsc::Receiver;
use std::net::UdpSocket;
use std::{fs};
use serde_json;
use std::thread;
use std::time;

pub struct RequestSender{
    addr: String,
    socket: UdpSocket,
    servers: Vec<String>,
    min_server: u16,
    min_server_load: u16
    // recepient_addr: String, 
}

const SERVERS_JSON : &str = "server.json";
impl RequestSender{
    pub fn new(addr: String) -> RequestSender {
        let servers_string = fs::read_to_string(SERVERS_JSON).expect("Could not read server json file");   
        let servers = serde_json::from_str(&servers_string).expect("Could not deserialize json");  
        RequestSender{
            addr: addr.clone(),
            servers: servers,
            min_server: 0,
            min_server_load: u16::MAX,
            socket: UdpSocket::bind(addr).expect("couldn't bind sender to address")
        }
    }


    pub fn init(&mut self) -> thread::JoinHandle<()>{
        return thread::spawn(move || { 
            let buf : [u8;2]= [2, 0];
            self.socket.set_read_timeout(Some(time::Duration::from_secs(4))).unwrap();
            loop
            {   self.min_server_load = u16::MAX;
                
                for (i, server) in self.servers.iter().enumerate()
                {
                    self.socket.send_to(&buf, server).expect("Failed to send pulse");
                    let mut recv_buf = [0; 2]; 
                    let recv_res = self.socket.recv_from(&mut recv_buf); 
                    match recv_res{
                        Ok((_,_)) => {
                            let val = u16::from_be_bytes(recv_buf);
                            if val < self.min_server_load{
                                self.min_server_load = val;
                                self.min_server = i as u16;
                            }
                        },

                        Err(_) => ()
                    };

                }
            }
         });
        
    }   
}