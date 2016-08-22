extern crate jack_client;
extern crate openal;
extern crate rosc;

use std::net::{UdpSocket, SocketAddrV4};
use std::str::FromStr;
use rosc::OscPacket;
use rosc::OscMessage;
use rosc::OscType::*;

mod ambibox;
use jack_client::client::*;

use openal::al;
use openal::alc;

struct OscSourceControl<'a> {
  handler: *mut ambibox::SourceHandler,
  client: &'a mut JackClient<ambibox::SourceHandler> 
}

impl<'a> OscSourceControl<'a> {
  pub fn new(client: &'a mut JackClient<ambibox::SourceHandler>, handler: *mut ambibox::SourceHandler) -> OscSourceControl<'a> {
    OscSourceControl {handler: handler, client: client}
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
  
  fn get_source(&self, source_str: &std::string::String) -> std::option::Option<&ambibox::Source> {
    unsafe { (*self.handler).sources.get::<std::string::String>(source_str) }
  }
  
  fn source_exists(&self, source_str: std::string::String) -> bool {
    unsafe { (*self.handler).sources.contains_key::<std::string::String>(&source_str) }
  }
  
  fn route_message(&mut self, msg: OscMessage) {
    match msg.addr.as_ref() {
      "/new_source" => self.new_source(msg),
      "/set_source_position" => self.set_source_position(msg),
      _ => {}
    }
  }
  
  fn new_source(&mut self, msg: OscMessage) {
    match msg.args {
      Some(arg) => {
        match arg[0] {
          String(ref source_str) => {
            match self.get_source(&source_str) {
              Some(source) => println!("Source already exists"),
              None => unsafe { (*self.handler).new_source(self.client, source_str.clone())}
            }
          },
          _ => {}
        }
      },
      None => println!("Missing source name argument")
    }
  }
  
  fn set_source_position(&self, msg: OscMessage) {
    match msg.args {
      Some(arg) => {
        match (&arg[0],&arg[1],&arg[2],&arg[3]) {
          (&String(ref source_str),&Float(x), &Float(y), &Float(z)) => {
            match self.get_source(&source_str) {
              Some(source) => source.set_position([x,y,z]),
              None => println!("No source found with name {}", source_str)
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
  let al_device = alc::Device::open(None).expect("Could not open device");
  let al_context = al_device.create_context(&[]).expect("Could not create context");
  al_context.make_current();
  
  let client_ref = &mut myclient;
  let myhandler: *mut ambibox::SourceHandler = ambibox::new_handler(client_ref,al_context);

  
  
  let mut osc_server = OscSourceControl::new(client_ref, myhandler);
  osc_server.run();
}

