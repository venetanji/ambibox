use jack_client::callbacks::*;
use jack_client::port::*;
use jack_client::client::*;

use openal::al;
use openal::alc;

use std::collections::HashMap;
use std::i16;

pub struct Source {
  al_buffers: Vec<al::Buffer>,
  al_source: al::Source,
  jack_port: JackPort
}

pub struct SourceHandler {
  pub sources: HashMap<String,Source>,
  al_context: alc::Context
}

impl JackHandler for SourceHandler {
  
  fn thread_init(&mut self) {
    self.al_context.make_current();
  }
  
  fn process(&mut self, n_frames:u32 ) -> JackControl {
    for source in self.sources.values_mut() {
      let current_slice = unsafe { source.jack_port.get_slice(n_frames)};
      let samples: Vec<i16> = (0..n_frames).map(|x| {
        let t: f32 = current_slice[x as usize];
        (t * (i16::MAX - 1) as f32) as i16
      }).collect();
    
      match source.al_source.get_buffers_processed() {
        0 => {
          match source.al_source.get_buffers_queued() {
            e @ 0 ... 1 => {
              unsafe { source.al_buffers[e].buffer_data(al::Format::Mono16, &samples, 48000 as al::ALsizei) }
              source.al_source.queue_buffer(&source.al_buffers[e]);
            },
            _ => source.al_source.play()
          } 
        }
        _ => {
          source.al_source.unqueue_buffer(&mut source.al_buffers[0]);
          unsafe { source.al_buffers[0].buffer_data(al::Format::Mono16, &samples, 48000 as al::ALsizei) }
          source.al_source.queue_buffer(&source.al_buffers[0]);
        }
      };
    }
    JackControl::Continue
  }
}

impl Source {
  pub fn set_position (&self, point: [f32; 3]) {
    self.al_source.set_position(point)
  }
}

impl SourceHandler {
  pub fn new_source(&mut self, myclient: &JackClient<SourceHandler>, port_name: String ) {
    let new_source = Source {
      al_buffers: vec![al::Buffer::gen(),al::Buffer::gen()],
      al_source: al::Source::gen(),
      jack_port: myclient.new_port().name(port_name.clone()).direction(Direction::Input).register().unwrap()
    };
    self.sources.insert(port_name,new_source);
  }
}

pub fn new_jack_client() -> JackClient<SourceHandler>  {
  JackClient::new().name("Ambibox")
}

pub fn new_handler(myclient: &mut JackClient<SourceHandler>, al_context: alc::Context) -> *mut SourceHandler {
  let new_handler = SourceHandler {
      sources: HashMap::new(),
      al_context: al_context
    };
  myclient.activate(new_handler).unwrap()
}


