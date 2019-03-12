//! A wrapper around a `std::sync::mpsc::sync_channel` that allows easy creation
//! of a dummy sender for tests, and doesn't error if the receiver hung up

pub use std::sync::mpsc::Receiver;
use std::sync::mpsc::{sync_channel, SyncSender};

pub fn new<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let (std_sender, std_receiver) = sync_channel(size);
    (Sender(std_sender), std_receiver)
}

#[cfg(test)]
pub fn dummy_sender<T>() -> Sender<T> {
    new(1).0
}

pub struct Sender<T>(SyncSender<T>);

impl<T> Sender<T> {
    pub fn send(&self, t: T) {
        let _ = self.0.send(t);
    }
}

impl<T> Clone for Sender<T>
where
    SyncSender<T>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
