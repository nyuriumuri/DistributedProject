
use std::fs::{File};
// use  std::sync::mpsc::Sender;
use std::net::{UdpSocket, SocketAddr, SocketAddrV4};
// use std::os::unix::net::SocketAddr;
use std::thread::{JoinHandle};
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
    fn to_bytes(self) -> [u8; 1024]{
        let mut bytes: [u8; 1024] = [0; 1024];
        let mode = self.mode;
        match mode{
            ElectionMsgMode::Ack => {
                bytes[0..3].clone_from_slice( &['A' as u8,'C' as u8,'K' as u8]); 
                bytes[3..self.data.len()+3].copy_from_slice(self.data.as_bytes());
            },
            ElectionMsgMode::Msg => {
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

            panic!("Invalid ElectionMsg byte array, GOT {:?}", bytes);
        }
    }
} 

pub struct RequestReceiver{
    addr: String,
    socket: Arc<Mutex<UdpSocket>>,
    sender: Arc<Mutex<Sender<([u8; 100], SocketAddr)>>>, 
    receiver:Arc<Mutex<Receiver<([u8; 100], SocketAddr)>>>,
    request_socket: Arc<Mutex<UdpSocket>>,
    load: Arc<Mutex<u128>>,
    index: usize, 
    election_ports: [String; 3],
    election_socket: Arc<Mutex<UdpSocket>>,
    rng_val: u32, 
    pub times_elected: u32,

}


impl RequestReceiver{
    pub fn new(addr: String, index: usize) -> RequestReceiver {
        let mut request_addr = SocketAddrV4::from_str(&addr).unwrap();
        request_addr.set_port(request_addr.port()+100); 
        let (sender, receiver) = channel::<([u8; 100], SocketAddr)>();
        let election_ports = ["192.168.1.114:9000".to_string(),"192.168.1.130:9001".to_string(),"192.168.1.130:9002".to_string()]; 
        RequestReceiver{
            addr: addr.clone(),
            socket: Arc::new(Mutex::new(UdpSocket::bind(addr).expect("couldn't bind sender to address"))),
            sender: Arc::new(Mutex::new(sender)), 
            receiver: Arc::new(Mutex::new(receiver)),
            request_socket: Arc::new(Mutex::new(UdpSocket::bind(request_addr).expect("Failed to bind to request addr"))), 
            load: Arc::new(Mutex::new(0)),
            index,
            election_ports: election_ports.clone(),
            election_socket: Arc::new(Mutex::new(UdpSocket::bind(&election_ports[index]).expect("Couldn't bindto election socket"))),
            rng_val: 0,
            times_elected: 0,

        }
    }

    pub fn listen(&self) -> JoinHandle<()>{
        println!("Listening on port {}", self.addr);
        let load_arc = self.load.clone();
        let socket_arc = self.socket.clone();
        let sender_arc  = self.sender.clone();
        return thread::spawn(move || loop{
               
                let sender = sender_arc.lock().unwrap();
                let mut buf = [0; 100]; 
                let src_addr: SocketAddr;    
                {
                let socket =  socket_arc.lock().unwrap();
                (_, src_addr) = (*socket).recv_from(&mut buf).expect("Didn't receive data");
                    drop(socket);
                 }
              
                if buf[0] == 2  // asking for load
                {
                    let reply_addr = src_addr;
          
                    let message = load_arc.lock().unwrap();
                    // println!("Load: {}",*message);
                    let message = (*message).to_be_bytes();
                    // println!("{:?}", message);
                    {
                        let socket =  socket_arc.lock().unwrap();
                        (*socket).send_to(&message, reply_addr).expect("Failed to send acknowledgement");
                        drop(socket)
                    }
               
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

    pub fn handle_requests(&self) -> JoinHandle<()>{
        let request_socket = self.request_socket.clone();
        let receiver = self.receiver.clone();
        let load_guard = self.load.clone();
        thread::spawn(move || loop{
            // read request from queue, send a reply, then sleep
            let (request, addr) = receiver.lock().unwrap().recv().unwrap();  
            let mut load = load_guard.lock().unwrap();
            *load -= 1; 
            drop(load);  // dropping mutex early as we no longer need it 
            println!("{}", String::from_utf8(request.to_vec()).unwrap());
            // let reply = String::from("REQ PROCESSED");
            request_socket.lock().unwrap().send_to(&request, addr).expect("Failed to send processed request");
            thread::sleep(Duration::from_micros(5000));
            
        })
    }

     // function for election in a new thread
     pub fn election(&mut self){
        
        self.rng_val = rand::thread_rng().gen();
        let election_socket = self.election_socket.clone();
        loop {  
           
        
            let mut slept = false;
            self.send_election_msg(self.rng_val);
            // loop till you get an election message
            let mut message = false;
            let mut addr: SocketAddr; 
            let mut buf = [0; 1024];
            while ! message {
       
                (_,addr) = {
                    // println!("Locking");
                    let lock = election_socket.lock().unwrap();
                    // println!("Locked");
                    lock.set_read_timeout(None).unwrap();

                    lock.recv_from(&mut buf).unwrap()
                };
                // println!("Unlocked");
                let msg = ElectionMsg::from(&buf);   
                if let ElectionMsgMode::Msg = msg.mode{   // if it's a message, handle it
                    slept = self.handle_election_msg(msg.data, addr); 
                    message = true;
        
                }
            }
            if slept{ break }

        }
        
    }

    fn handle_election_msg(&mut self, data: String, addr: SocketAddr) -> bool {
        let incoming_rng = data.trim_matches(char::from(0)).parse::<u32>().expect(format!("Invalid u32 data {}", data).as_str());
        println!("Incoming RNG: {}, My RNG: {}", incoming_rng, self.rng_val);
        let sent_rng: u32 = if incoming_rng > self.rng_val { incoming_rng } else { self.rng_val }; 
        let socket = self.election_socket.clone();
        {
            let ack_reply = ElectionMsg{mode:  ElectionMsgMode::Ack, data: sent_rng.to_string()}.to_bytes();
            let guard = socket.lock().unwrap();
            // println!("Sending ACK");
            guard.send_to(&ack_reply.clone(), addr).unwrap();
            // println!("ACK");
        }
  
        if incoming_rng == self.rng_val {
            // println!("LOCKING A");
             let a =  self.election_socket.clone();
             let _a = a.lock().unwrap(); 
            //  println!("LOCKING B");

             let b = self.request_socket.clone();
             let _b = b.lock().unwrap();
            //  println!("LOCKING C");

             let c =self.socket.clone();
             let _c = c.lock().unwrap(); 
            println!("Sleeping");
            thread::sleep(Duration::from_secs(60));  // fall for 1 min
            println!("Woke Up");
            self.rng_val = rand::thread_rng().gen();
            self.times_elected +=1; 
            return true;

        
        }else if incoming_rng > self.rng_val {
                self.rng_val = rand::thread_rng().gen();
                self.send_election_msg(sent_rng); 
            } else{
            self.send_election_msg(sent_rng); 
        }
        return false;
        

    }
    fn send_election_msg(&self, rng: u32){
        println!("SENDING {}", rng);
        let mut sent = false;
        let message =  ElectionMsg{mode: ElectionMsgMode::Msg, data: rng.to_string() }.to_bytes();
        // println!("Message: {:?}", message);
        let socket = self.election_socket.clone();     
            
        let mut i = self.index+1; 
        while !sent {                                           // loop until ack is received
            let mut buf: [u8; 1024] = [0; 1024];        
            let read_res = {
                let socket_guard = socket.lock().unwrap();     
                // println!("sending");
                socket_guard.send_to( &message.clone(), self.election_ports[i%3].clone()).unwrap();
                // println!("sent");
                socket_guard.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
                socket_guard.peek_from(&mut buf)

            };
            
            match read_res {
                    Ok(_) => {
                        // println!("OK");
                        sent = true;
                        }
                        
                    
                    Err(_) => {
                        // println!("Err");
                        i+=1;    // transmit to next server if recv failed
                        i%=3;
                        if i == self.index { i+=1}
                    }
            }
        }
        
    }


}