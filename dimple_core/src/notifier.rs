use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Notifier<T: Send> {
    subs: Arc<Mutex<Vec<Box<dyn Fn(&T) + Send>>>>,
}

impl <T: Send> Notifier<T> {
    pub fn new() -> Self {
        Self {
            subs: Default::default(),
        }
    }

    pub fn on_notify(&self, l: Box<dyn Fn(&T) + Send>) {
        self.subs.lock().unwrap().push(l);
    }

    pub fn notify(&self, arg: &T) {
        for sub in self.subs.lock().unwrap().iter() {
            sub(arg);
        }
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
        notifier.on_notify(Box::new(move |lie| {
            println!("{} moof", lie);
            COUNT.fetch_add(1, atomic::Ordering::Relaxed);
        }));
        let notifier1 = notifier.clone();
        let t = std::thread::spawn(move || {
            notifier.notify(&"dear rosemark".to_string());
        });
        notifier1.notify(&"hurk hurk hurk".to_string());
        t.join().unwrap();
        assert!(COUNT.load(atomic::Ordering::Relaxed) == 2);
    }
}
