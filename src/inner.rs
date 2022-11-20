use std::{
  collections::VecDeque,
  sync::{Condvar, Mutex},
};

/// Inner data sent by [`Sender`] and received by [`Receiver].
///
/// [`Sender`]: struct.Sender
/// [`Receiver]: struct.Receiver
#[derive(Debug)]
pub(crate) struct Inner<T> {
  pub(crate) queue: Mutex<VecDeque<T>>,
  pub(crate) available: Condvar,
}

impl<T> Inner<T> {
  pub(crate) fn new() -> Self {
    Self {
      queue: Mutex::default(),
      available: Condvar::new(),
    }
  }
}
