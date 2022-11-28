
use std::fs::{File};
// use  std::sync::mpsc::Sender;
use std::net::{UdpSocket, SocketAddr, SocketAddrV4};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{thread};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{ Sender, Receiver, channel};
use std::str::FromStr;
use std::io::Write;
//use rand::prelude::{random};

pub struct RequestReceiver{
    addr: String,
    socket: Arc<Mutex<UdpSocket>>,
    sender: Arc<Mutex<Sender<([u8; 100], SocketAddr)>>>, 
    receiver:Receiver<([u8; 100], SocketAddr)>,
    request_socket: UdpSocket,
    load: Arc<Mutex<u16>>,

}


impl RequestReceiver{
    pub fn new(addr: String) -> RequestReceiver {
        let mut request_addr = SocketAddrV4::from_str(&addr).unwrap();
        request_addr.set_port(request_addr.port()+100); 
        let (sender, receiver) = channel::<([u8; 100], SocketAddr)>();
        RequestReceiver{
            addr: addr.clone(),
            socket: Arc::new(Mutex::new(UdpSocket::bind(addr).expect("couldn't bind sender to address"))),
            sender: Arc::new(Mutex::new(sender)), 
            receiver: receiver,
            request_socket: UdpSocket::bind(request_addr).expect("Failed to bind to request addr"), 
            load: Arc::new(Mutex::new(0))

        }
    }

    pub fn listen(&self) -> JoinHandle<()>{
        println!("Listening on port {}", self.addr);
        let load_arc = self.load.clone();
        let socket_arc = self.socket.clone();
        let sender_arc  = self.sender.clone();
        return thread::spawn(move || loop{
                let socket =  socket_arc.lock().unwrap();
                let sender = sender_arc.lock().unwrap();
                let mut buf = [0; 100];    
                let (_, src_addr) = (*socket).recv_from(&mut buf).expect("Didn't receive data");
                // let buf_str = str::from_utf8(&buf[..]).unwrap();
                // println!("{}", buf_str);
                // println!("Got a message from {}", src_addr);
                if buf[0] == 2  // asking for load
                {
                    let reply_addr = src_addr;
          
                    let message = load_arc.lock().unwrap();
                    // println!("Load: {}",*message);
                    let message = (*message).to_be_bytes();
                    // println!("{:?}", message);
                    (*socket).send_to(&message, reply_addr).expect("Failed to send acknowledgement");
                }
                else if  buf[0] == 1 { // ack
                    
                }
                else {  // got a request from the client
                    let mut load = load_arc.lock().unwrap();
                    *load += 1;
                    drop(load);
                    sender.send((buf, src_addr)).expect("Failed to pass request in queue"); // add request to queue to be handled
   
                }
        });
    }

    pub fn log_stats(&self) -> JoinHandle<()>{
        let load_arc = self.load.clone();

        let mut file = File::create({
            let fname = self.addr.clone();
            
            format!("{}.txt", fname.replace(":", "-"))
        }).unwrap();

        return thread::spawn(move || loop{
            let load_val = {
                let load   = load_arc.lock().unwrap();
                *load 
            };

    
           if let Err(_) =  writeln!(file, "{}", load_val)
           {
             ()
           }
           thread::sleep(Duration::from_secs(1)); 
        });
    }

    pub fn handle_requests(&self){
        loop{
            // read request from queue, send a reply, then sleep
            let (request, addr) = self.receiver.recv().unwrap();  
            let mut load = self.load.lock().unwrap();
            *load -= 1; 
            drop(load);  // dropping mutex early as we no longer need it 
            println!("{}", String::from_utf8(request.to_vec()).unwrap());
            // let reply = String::from("REQ PROCESSED");
            self.request_socket.send_to(&request, addr).expect("Failed to send processed request");
            thread::sleep(Duration::from_micros(1000));
            
        }
    }

}