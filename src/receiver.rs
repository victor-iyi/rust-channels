use std::{
  fmt,
  sync::Arc,
  time::{Duration, Instant},
};

use crate::{
  error::{RecvError, RecvTimeoutError, TryRecvError},
  inner::Inner,
  iterators::{Iter, TryIter},
};

/// The receiving half of Rust's [`channel`] (or [`sync_channel`]) type. This
/// half can only be owned by one thread.
///
/// Messages sent to the channel can be retrieved by [`recv`].
///
/// [`channel`]: fn.channel
/// [`sync_channel]: fn.sync_channel
/// [`recv`]: struct.Receiver#method.recv
pub struct Receiver<T> {
  inner: Arc<Inner<T>>,
}

// The receiver port can be sent from place to place, so long as it
// is not used to receive non-sendable things.
unsafe impl<T: Send> Send for Receiver<T> {}

// impl<T> !Sync for Receiver<T> {}

impl<T> fmt::Debug for Receiver<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Receiver").finish_non_exhaustive()
  }
}
impl<T> Receiver<T> {
  pub(super) fn new(inner: Arc<Inner<T>>) -> Self {
    Self { inner }
  }

  /// Attempts to return a pending value on this receiver without blocking.
  ///
  /// This method will never block the caller in order to wait for data to become
  /// available. Instead, this will always return immediately with a possible
  /// option of pending data on the channel.
  ///
  /// This is useful for a flavor of "optimistic check" before deciding to block
  /// on a receiver.
  ///
  /// Compared with `[recv]`, this function has two failure cases instead of one
  /// (one for disconnection, one for any empty buffer).
  ///
  /// [`recv`]: struct.Receiver#method.recv
  ///
  /// # Examples
  ///
  /// ```rust
  /// use channel::{Receiver, channel};
  ///
  /// let (_, receiver): (_, Receiver<i32>) = channel();
  ///
  /// assert!(receiver.try_recv().is_err());
  /// ```
  pub fn try_recv(&self) -> Result<T, TryRecvError> {
    let mut queue = self.inner.queue.lock().unwrap();
    queue.pop_front().ok_or(TryRecvError::Disconnected)
  }

  /// Attempts to wait for a value on this receiver, returning an error if the
  /// corresponding channel has hung up.
  ///
  /// This function will always block the current thread if there is no data
  /// available and it's possible for more data to be sent. Once a message
  /// is sent to the corresponding [`Sender`] (or [`SyncSender`]), then this
  /// receiver will wake up and return that message.
  ///
  /// If the corresponding [`Sender`] has disconnected, or it disconnects while
  /// this call is blocking, this call will wake up and return [`Err`] to
  /// indicate that no more messages can ever be received on this channel.
  /// However, since channels are buffered, messages sent before the disconnect
  /// will still be properly received.
  ///
  /// [`Sender`]: struct.Sender
  /// [`SyncSender`]: struct.SyncSender
  /// [`Err`]: std::result::Result::Err
  /// [`Ok`]: std::result::Result::Ok
  ///
  /// # Example
  ///
  /// ```rust
  /// use std::thread;
  /// use channel::channel;
  ///
  /// let (mut send, mut recv) = channel();
  /// let handle = thread::spawn(move || {
  ///   send.send(1u8).unwrap();
  /// });
  ///
  /// handle.join().unwrap();
  ///
  /// assert_eq!(Ok(1), recv.recv());
  /// ```
  pub fn recv(&self) -> Result<T, RecvError> {
    let mut queue = self.inner.queue.lock().unwrap();
    // Block/sleep till you receive some data.
    loop {
      let data = queue.pop_front();

      match data {
        Some(d) => return Ok(d),
        None => {
          queue = self.inner.available.wait(queue).unwrap();
        }
      }
    }

    // Unreachable.
    // Err(RecvError)
  }

  /// Returns an iterator that will block waiting for messages, but never
  /// [`panic!`]. It will return [`None`] when the channel has hung up.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use channel::channel;
  /// use std::thread;
  ///
  /// let (send, recv) = channel();
  ///
  /// thread::spawn(move || {
  ///   send.send(1).unwrap();
  ///   send.send(2).unwrap();
  ///   send.send(3).unwrap();
  /// });
  ///
  /// let mut iter = recv.iter();
  /// assert_eq!(iter.next(), Some(1));
  /// assert_eq!(iter.next(), Some(2));
  /// assert_eq!(iter.next(), Some(3));
  /// assert_eq!(iter.next(), None);
  /// ```
  pub fn iter(&self) -> Iter<'_, T> {
    Iter { rx: self }
  }

  /// Returns an iterator that will attempt to yield all pending values.
  /// It will return `None` if there are no more pending values or if the
  /// channel has hung up. The iterator will never [`panic!`] or block the
  /// user by waiting for values.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use std::{thread, time::Duration};
  ///
  /// use channel::channel;
  ///
  /// let (sender, receiver) = channel();
  ///
  /// // Nothing is in the buffer yet.
  /// assert!(receiver.try_iter().next().is_none());
  ///
  /// thread::spawn(move || {
  ///   thread::sleep(Duration::from_secs(1));
  ///   sender.send(1).unwrap();
  ///   sender.send(2).unwrap();
  ///   sender.send(3).unwrap();
  /// });
  ///
  /// // Nothing is in the buffer yet.
  /// assert!(receiver.try_iter().next().is_none());
  ///
  /// // Block for 2 seconds.
  /// thread::sleep(Duration::from_secs(2));
  ///
  /// let mut iter = receiver.try_iter();
  /// assert_eq!(iter.next(), Some(1));
  /// assert_eq!(iter.next(), Some(2));
  /// assert_eq!(iter.next(), Some(3));
  /// assert_eq!(iter.next(), None);
  /// ```
  pub fn try_iter(&self) -> TryIter<'_, T> {
    TryIter { rx: self }
  }

  /// Attempts to wait for a value on this receiver, returning an error if the
  /// corresponding channel has hung up, or if it waits more than `timeout`.
  ///
  /// This function will always block the current thread if there is no data
  /// available and it's possible for more data to be sent (at least one sender
  /// still exists). Once a message is sent to the corresponding [`Sender`]
  /// (or [`SyncSender`]), this receiver will wake up and return that
  /// message.
  ///
  /// If the corresponding [`Sender`] has disconnected, or it disconnects while
  /// this call is blocking, this call will wake up and return [`Err`] to
  /// indicate that no more messages can ever be received on this channel.
  /// However, since channels are buffered, messages sent before the disconnect
  /// will still be properly received.
  ///
  /// # Known Issues
  ///
  /// There is currently a known issue (see [`#39364`]) that causes `recv_timeout`
  /// to panic unexpectedly with the following example:
  ///
  /// ```no_run
  /// use std::sync::mpsc::channel;
  /// use std::thread;
  /// use std::time::Duration;
  ///
  /// let (tx, rx) = channel::<String>();
  ///
  /// thread::spawn(move || {
  ///     let d = Duration::from_millis(10);
  ///     loop {
  ///         println!("recv");
  ///         let _r = rx.recv_timeout(d);
  ///     }
  /// });
  ///
  /// thread::sleep(Duration::from_millis(100));
  /// let _c1 = tx.clone();
  ///
  /// thread::sleep(Duration::from_secs(1));
  /// ```
  ///
  /// [`#39364`]: https://github.com/rust-lang/rust/issues/39364
  ///
  /// # Examples
  ///
  /// Successfully receiving value before encountering timeout:
  ///
  /// ```no_run
  /// use std::thread;
  /// use std::time::Duration;
  /// use std::sync::mpsc;
  ///
  /// let (send, recv) = mpsc::channel();
  ///
  /// thread::spawn(move || {
  ///     send.send('a').unwrap();
  /// });
  ///
  /// assert_eq!(
  ///     recv.recv_timeout(Duration::from_millis(400)),
  ///     Ok('a')
  /// );
  /// ```
  ///
  /// Receiving an error upon reaching timeout:
  ///
  /// ```no_run
  /// use std::thread;
  /// use std::time::Duration;
  /// use std::sync::mpsc;
  ///
  /// let (send, recv) = mpsc::channel();
  ///
  /// thread::spawn(move || {
  ///     thread::sleep(Duration::from_millis(800));
  ///     send.send('a').unwrap();
  /// });
  ///
  /// assert_eq!(
  ///     recv.recv_timeout(Duration::from_millis(400)),
  ///     Err(mpsc::RecvTimeoutError::Timeout)
  /// );
  /// ```
  pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
    match self.try_recv() {
      Ok(result) => Ok(result),
      Err(TryRecvError::Disconnected) => Err(RecvTimeoutError::Disconnected),
      Err(TryRecvError::Empty) => match Instant::now().checked_add(timeout) {
        Some(deadline) => self.recv_deadline(deadline),
        // So far in the future that it's practically the same as waiting indefinitely.
        None => self.recv().map_err(RecvTimeoutError::from),
      },
    }
  }

  pub fn recv_deadline(&self, _deadline: Instant) -> Result<T, RecvTimeoutError> {
    unimplemented!()
  }
}
