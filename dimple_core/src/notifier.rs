use std::sync::{Arc, Mutex};

use threadpool::ThreadPool;

#[derive(Clone)]
pub struct Notifier<Event: Send + Sync + Clone + 'static> {
    subscribers: Arc<Mutex<Vec<Box<dyn FnMut(Event) -> () + Send + 'static>>>>,
    threadpool: ThreadPool,
}

impl <Event: Send + Sync + Clone + 'static> Notifier<Event> {
    pub fn new() -> Self {
        Self {
            subscribers: Default::default(),
            threadpool: ThreadPool::new(1),
        }
    }

    pub fn notify(&self, event: Event) {
        let subs = self.subscribers.clone();
        self.threadpool.execute(move || {
            for sub in subs.lock().unwrap().iter_mut() {
                sub(event.clone());
            }
        });
    }

    pub fn observe(&self, callback: impl FnMut(Event) -> () + Send + 'static) {
        self.subscribers.lock().unwrap().push(Box::new(callback));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{self, AtomicU8};

    use crate::notifier::Notifier;

    #[test]
    fn test() {
        let notifier = Notifier::<String>::new();
        static COUNT: AtomicU8 = AtomicU8::new(0);
        notifier.observe(move |event| {
            println!("{} moof", event);
            COUNT.fetch_add(1, atomic::Ordering::Relaxed);
        });
        let notifier1 = notifier.clone();
        let t = std::thread::spawn(move || {
            notifier.notify("dear rosemark".to_string());
        });
        notifier1.notify("hurk hurk hurk".to_string());
        t.join().unwrap();
        assert!(COUNT.load(atomic::Ordering::Relaxed) == 2);
    }
}
