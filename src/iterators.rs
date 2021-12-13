use crate::Receiver;

/// An iterator over messages on a [`Receiver`], created by [`iter`].
///
/// This iterator will block whenever [`next`] is called, waiting for a new
/// message, and [`None`] will be returned when the corresponding channel has
/// hung up.
///
/// [`iter`]: Receiver::iter
/// [`next`]: Iterator::next
///
/// # Examples
///
/// ```rust,ignore
/// use channel::{channel};
/// use std::thread;
///
/// let (send, recv) = channel();
///
/// thread::spawn(move || {
///   send.send(1u8).unwrap();
///   send.send(2u8).unwrap();
///   send.send(3u8).unwrap();
/// });
///
/// for x in recv.iter() {
///   println!("Got: {}", x);
/// }
/// ```
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    rx: &'a Receiver<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

#[derive(Debug)]
pub struct TryIter<'a, T: 'a> {
    rx: &'a Receiver<T>,
}

impl<'a, T> Iterator for TryIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

impl<'a, T> IntoIterator for &'a Receiver<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

impl<T> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { rx: self }
    }
}

#[derive(Debug)]
pub struct IntoIter<T> {
    rx: Receiver<T>,
}
