use std::{sync::{mpsc::{channel, Receiver, Sender}, Arc, RwLock}, thread};

#[derive(Clone)]
pub struct Notifier<Event: Send + Sync + Clone + 'static> {
    senders: Arc<RwLock<Vec<Sender<Event>>>>,
}

impl <Event: Send + Sync + Clone + 'static> Notifier<Event> {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn notify(&self, event: Event) {
        self.senders.read().unwrap().iter().for_each(|tx| {
            tx.send(event.clone()).unwrap();
        });
    }

    pub fn observer(&self) -> Receiver<Event> {
        let (tx, rx) = channel();
        self.senders.write().unwrap().push(tx);
        rx
    }

    pub fn observe(&self, mut callback: impl FnMut(Event) -> () + Send + 'static) {
        let rx = self.observer();
        thread::spawn(move || {
            rx.iter().for_each(|e| callback(e));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::Notifier;

    #[test]
    fn test() {
        let notifier = Notifier::<String>::new();
        let rx = notifier.observe(move |e| println!("{}", e));
        notifier.notify("oh wow".to_string());
        notifier.notify("oh jeez".to_string());
        notifier.notify("oh golly".to_string());
        notifier.notify("oh gosh".to_string());
    }
}
