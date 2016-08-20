use jack_client::callbacks::*;
use jack_client::port::*;
use jack_client::client::*;

use openal::al;
use openal::alc;

use std::i16;

pub struct SourceHandler {
  al_buffers: Vec<al::Buffer>,
  al_source: al::Source,
  jack_port: JackPort
}

impl JackHandler for SourceHandler {
  
  fn process(&mut self, n_frames:u32 ) -> JackControl {
    let current_slice = unsafe { self.jack_port.get_slice(n_frames)};
    let samples: Vec<i16> = (0..n_frames).map(|x| {
      let t: f32 = current_slice[x as usize];
      (t * (i16::MAX - 1) as f32) as i16
    }).collect();
    
    match self.al_source.get_buffers_processed() {
      0 => {
        match self.al_source.get_buffers_queued() {
          e @ 0 ... 1 => {
            unsafe { self.al_buffers[e].buffer_data(al::Format::Mono16, &samples, 48000 as al::ALsizei) }
            self.al_source.queue_buffer(&self.al_buffers[e]);
          },
          _ => self.al_source.play()
        } 
      }
      _ => {
        self.al_source.unqueue_buffer(&mut self.al_buffers[0]);
        unsafe { self.al_buffers[0].buffer_data(al::Format::Mono16, &samples, 48000 as al::ALsizei) }
        self.al_source.queue_buffer(&self.al_buffers[0]);
      }
    };
    
    JackControl::Continue
    //println!("> {} => {}", current_slice[0], samples[0]);
    //unsafe { self.al_buffer.buffer_data(al::Format::Mono16, &samples, 48000 as al::ALsizei) };
  }
}

impl SourceHandler {
  pub fn set_position (&self, point: [f32; 3]) {
    self.al_source.set_position(point)
  }
}

pub fn new_jack_client() -> JackClient<SourceHandler>  {
  init_al();
  JackClient::new().name("Ambibox")
}

pub fn new_source_handler(myclient: &mut JackClient<SourceHandler>, port_name: &str ) -> *mut SourceHandler {
  let new_handler = SourceHandler {
      al_buffers: vec![al::Buffer::gen(),al::Buffer::gen()],
      al_source: al::Source::gen(),
      jack_port: myclient.new_port().name(port_name).direction(Direction::Input).register().unwrap()
    };
  myclient.activate(new_handler).unwrap()
}

fn init_al() {
  let al_device = alc::Device::open(None).expect("Could not open device");
  let al_context = al_device.create_context(&[]).expect("Could not create context");
  al_context.make_current();
  al::listener::set_position([0.0,0.0,0.0]);
}

