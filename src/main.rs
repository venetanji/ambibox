extern crate jack_client;
extern crate openal;
extern crate rosc;

use std::net::{UdpSocket, SocketAddrV4};
use std::str::FromStr;
use rosc::OscPacket;
use rosc::OscMessage;
use rosc::OscType::*;
use std::collections::HashMap;

mod ambibox;
use jack_client::client::*;

struct OscSourceControl<'a> {
  sources: HashMap<&'a str,*mut ambibox::SourceHandler>,
  client: &'a mut JackClient<ambibox::SourceHandler> 
}

impl<'a> OscSourceControl<'a> {
  pub fn new(client: &'a mut JackClient<ambibox::SourceHandler>) -> OscSourceControl<'a> {
    OscSourceControl {sources: HashMap::new(), client: client}
  }
  
  pub fn run(&mut self) {
    let addr =SocketAddrV4::from_str("127.0.0.1:9001").unwrap();
    let sock = UdpSocket::bind(addr).unwrap();
    println!("Listening to {}", addr);

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
      match sock.recv_from(&mut buf) {
        Ok((size, addr)) => {
          println!("Received packet with size {} from: {}", size, addr);
          let packet = rosc::decoder::decode(&buf[..size]).unwrap();
          match packet {
            OscPacket::Message(msg) => self.route_message(msg),
            OscPacket::Bundle(bundle) => {}
          }
        }
        Err(e) => {
          println!("Error receiving from socket: {}", e);
          break;
        }
      }
    }
  }
  
  fn route_message(&mut self, msg: OscMessage) {
    match msg.addr.as_ref() {
      "/new_source" => self.new_source_route(msg),
      "/set_source_position" => self.set_source_position_route(msg),
      _ => {}
    }
  }
  
  fn new_source_route(&mut self, msg: OscMessage) {
    match msg.args {
      Some(arg) => {
        match &arg[0] {
          &String(ref source_str) => {
            let source = self.sources.get::<str>(&source_str);
            match source {
              Some(s) => println!("A source with name '{}' already exists", &source_str),
              None => {
                self.sources.insert(&source_str, ambibox::new_source_handler(self.client, &source_str));
              }
            }
          },
          _ => {}
        }
      },
      None => println!("Missing source name argument")
    }
  }
  
  fn set_source_position_route(&self, msg: OscMessage) {
    match msg.args {
      Some(arg) => {
        match (&arg[0],&arg[1],&arg[2],&arg[3]) {
          (&String(ref source_str),&Float(x), &Float(y), &Float(z)) => {
            let source = self.sources.get::<str>(&source_str);
            match source {
              Some(ptr) => unsafe { (**ptr).set_position([x,y,z])},
              None => println!("Source not found")
            }
            
          },
          (_,_,_,_) => {}
        }
      },
      None => {}
    }
  }
}

fn main() {
  let mut myclient: JackClient<ambibox::SourceHandler>  = ambibox::new_jack_client();
  myclient = myclient.connect().unwrap();
  let client_ref = &mut myclient;
  let mut osc_server = OscSourceControl::new(client_ref);
  osc_server.run();
}

