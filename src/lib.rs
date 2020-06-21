use std::sync::mpsc::{channel, Sender, SendError};
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct ThreadPool {
  _handles: Vec<std::thread::JoinHandle<()>>,
  sender: Sender<Box<dyn Fn() + Send>>,
  wait: Arc<AtomicU32>,
}

impl ThreadPool {

  pub fn new(num_threads: u8) -> Self {
    let (sender, receiver) = channel::<Box<dyn Fn() + Send>>();
    let receiver = Arc::new(Mutex::new(receiver));

    let wait = AtomicU32::new(0);
    let nref = Arc::new(wait);
    let mut _handles = vec![];

    for _ in 0..num_threads {
      let clone = receiver.clone();
      let counter = nref.clone();
      let handle = std::thread::spawn(move || loop {
        counter.fetch_add(1, Ordering::SeqCst);

        let work = match clone.lock().unwrap().recv() {
          Ok(work) => work,
          Err(_) => break,
        };

        println!("Starting work...");
        work();
        println!("Finishing work...");
        counter.fetch_sub(1, Ordering::SeqCst);
        let t = counter.clone();
        println!("Val: {:?}", t.load(Ordering::SeqCst));
      });

      _handles.push(handle);
    }


    Self { _handles, sender, wait: nref }
  }

  pub fn execute<T: Fn() + Send + 'static>(&self, work: T) {
    self.sender.send(Box::new(work)).unwrap();
  }

  pub fn wait(self) {
    // @todo - this doesn't __really__ work, it just waits
    // forever
    while self.wait.load(Ordering::SeqCst) != 0 {
      println!("Processing");
    }
  }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
      let pool = ThreadPool::new(10);
      pool.execute(|| println!("Hello world"));
      pool.execute(|| println!("Hello world"));
      pool.execute(|| println!("Hello world"));
      pool.execute(|| println!("Hello world"));
      pool.execute(|| println!("Hello world"));
      pool.execute(|| println!("Hello world"));

      pool.wait()
    }
}
