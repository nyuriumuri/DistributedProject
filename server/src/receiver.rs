
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


use rand::Rng;
//use rand::prelude::{random};

enum ElectionMsgMode{
    Ack,
    Msg,
}

struct ElectionMsg{
    mode: ElectionMsgMode,
    data: String
}

impl ElectionMsg{
    fn toBytes(&self) -> [u8; 1024]{
        let mut bytes: [u8; 1024] = [0; 1024];
        match self.mode{
            Ack => {
                bytes[0..3].clone_from_slice( &['A' as u8,'C' as u8,'K' as u8]); 
                bytes[3..self.data.len()+3].copy_from_slice(self.data.as_bytes());
            },
            Msg => {
                bytes[0..3].clone_from_slice( &['M' as u8,'S' as u8,'G' as u8]); 
                bytes[3..self.data.len()+3].copy_from_slice(self.data.as_bytes());
            }
        }
        bytes
    }
}

impl From<&[u8;1024]> for ElectionMsg {
    fn from(bytes:&[u8;1024]) -> Self {

        if &bytes[0..3] == &['A' as u8, 'C' as u8, 'K' as u8][..] {
            return ElectionMsg{ mode:ElectionMsgMode::Ack, data:String::from_utf8(bytes[3..].to_vec()).expect("Could not convert byte array to ElectionMsg Data") };
        }else if &bytes[0..3] == &['M' as u8, 'S' as u8, 'G' as u8][..]{

            return ElectionMsg{ mode:ElectionMsgMode::Msg, data:String::from_utf8(bytes[3..].to_vec()).expect("Could not convert byte array to ElectionMsg Data") };

        }else{

            panic!("Invalid ElectionMsg byte array");
        }
    }
} 

pub struct RequestReceiver{
    addr: String,
    socket: Arc<Mutex<UdpSocket>>,
    sender: Arc<Mutex<Sender<([u8; 100], SocketAddr)>>>, 
    receiver:Receiver<([u8; 100], SocketAddr)>,
    request_socket: UdpSocket,
    load: Arc<Mutex<u16>>,
    index: usize, 
    election_ports: [String; 3],
    election_socket: Arc<Mutex<UdpSocket>>

}


impl RequestReceiver{
    pub fn new(addr: String, index: usize) -> RequestReceiver {
        let mut request_addr = SocketAddrV4::from_str(&addr).unwrap();
        request_addr.set_port(request_addr.port()+100); 
        let (sender, receiver) = channel::<([u8; 100], SocketAddr)>();
        let  election_ports = ["127.0.0.1:9000".to_string(),"127.0.0.1:9001".to_string(),"127.0.0.1:9002".to_string()]; 
        RequestReceiver{
            addr: addr.clone(),
            socket: Arc::new(Mutex::new(UdpSocket::bind(addr).expect("couldn't bind sender to address"))),
            sender: Arc::new(Mutex::new(sender)), 
            receiver: receiver,
            request_socket: UdpSocket::bind(request_addr).expect("Failed to bind to request addr"), 
            load: Arc::new(Mutex::new(0)),
            index,
            election_ports,
            election_socket: Arc::new(Mutex::new(UdpSocket::bind(election_ports[index]).expect("Couldn't bindto election socket")))

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

     // function for election in a new thread
     pub fn election(&self) -> JoinHandle<()> {
        let addr_array = ["127.0.0.1:9000","127.0.0.1:9001","127.0.0.1:9002"];

        let index = self.index.clone();

        let mut buf = [0; 1024]; 

        let mut rng : i8 = rand::thread_rng().gen_range(1..3);

        let rng = rng.to_string();  


        return thread::spawn(move || loop {

                std::thread::sleep(std::time::Duration::from_secs(30)); // sleep for 30 seconds
                let mut rng : i8 = rand::thread_rng().gen_range(0..3); // generate random number from 0 to 2

                let rng = rng.to_string();   // convert random number to string
                let socket1 = UdpSocket::bind(addr_array[index]).unwrap(); // bind socket to port 9000 and store in socket1 variable


                
                loop {

                    let sent_to_next = false;  
                    let i = 1;

                    socket1.send_to(rng.as_bytes(), addr_array[(index+1)%3]).unwrap(); // send random number to port 9001
                    socket1.send_to(rng.as_bytes(), addr_array[(index+2)%3]).unwrap(); // send random number to port 9001

                    let (amt, src) = socket1.recv_from(&mut buf).unwrap(); // receive message from sender thread and store in amt and src variables
                    // println!("{}: {}", src, str::from_utf8(&buf[..amt]).unwrap()); // print message from sender thread to console

                    // print buf to console
                    let buf_str = std::str::from_utf8(&buf[..amt]).unwrap().clone();
                    // clone buf_str to index_0 to store index of leader

                    
                    std::thread::sleep(std::time::Duration::from_secs(5)); // sleep for 5 seconds
            }

        });
    }

    fn handle_election_msg(&self, data: String){
        // tbd

    }
    fn send_election_msg(&self, rng: u32){

        let mut sent = false;
        let message =  ElectionMsg{mode: ElectionMsgMode::Ack, data: rng.to_string() }.toBytes();
        let socket = self.election_socket.clone();     
        let socket_guard = socket.lock().unwrap();     
        let mut i = self.index+1; 
        while !sent {
            
            socket_guard.send_to( &message.clone(), self.election_ports[i]);
            socket_guard.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
            let buf: [u8; 1024] = [0; 1024];
            let read_res = socket_guard.recv_from(&mut buf);
            match read_res {
                    Ok(_) => {
                        let msg = ElectionMsg::from(&buf);
                        if let ElectionMsgMode::Msg = msg.mode{
                            self.handle_election_msg(msg.data); 
                        }else {
                            sent = true; 
                        }
                        
                    }
                    Err(_) => i+=1   
            }
        }
        
    }


}