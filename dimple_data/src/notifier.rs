use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Notifier<Event> {
    subscribers: Arc<Mutex<Vec<Box<dyn FnMut(Event) -> ()>>>>,
}

impl <Event: Clone> Notifier<Event> {
    pub fn new() -> Self {
        Self {
            subscribers: Default::default(),
        }
    }

    pub fn notify(&self, event: Event) {
        for sub in self.subscribers.lock().unwrap().iter_mut() {
            sub(event.clone());
        }
    }

    pub fn observe(&self, callback: impl FnMut(Event) -> () + 'static) {
        self.subscribers.lock().unwrap().push(Box::new(callback));
    }
}