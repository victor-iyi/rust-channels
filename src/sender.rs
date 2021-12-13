use std::{fmt, sync::Arc};

use crate::{
    error::{SendError, TrySendError},
    Inner,
};

/// The sending-half of Rust's asynchronous [`channel`] type. This half can only
/// be owned by one thread, but it can be cloned to send to other threads.
///
/// Messages can be sent through this channel with [`send`].
///
/// [`channel`]: fn.channel
/// [`send`]: struct.Sender#method.send
///
/// # Example
///
/// ```rust,ignore
/// use channel::channel;
///
/// let (mut sender, mut receiver) = channel();
/// let mut sender2 = sender.clone();
///
/// // First thread owns `sender`.
/// std::thread::spawn(move || {
///   sender.send(1).unwrap();
/// });
///
/// // Second thread owns `sender2`.
/// std::thread::spawn(move || {
///   sender2.send(2).unwrap();
/// });
///
/// let msg = receiver.recv().unwrap();
/// let msg2 = receiver.recv().unwrap();
///
/// assert_eq!(msg + msg2, 3);
/// ```
pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
    }
}

// The send port can be sent from place to place, so long as it
// is not used to send non-senable things.
unsafe impl<T: Send> Send for Sender<T> {}

// impl<T> !Sync for Sender<T> {}

impl<T> Sender<T> {
    pub(super) fn new(inner: Arc<Inner<T>>) -> Self {
        Self { inner }
    }

    /// Attempts to send a value on this channel, returning it back if it could
    /// not be sent.
    ///
    /// A successful send occurs when it is determined that the other end of the
    /// channel has not hug up already. An unsuccessful send would be one where
    /// the corresponding receiver has already been deallocated. Note that a
    /// return value of [`Err`] means that the data will be received, but a return
    /// value of [`Ok`] does *not* mean that the data will be received. It is
    /// possible for the corresponding receiver to hang up immediately after this
    /// function returns [`Ok`].
    ///
    /// This method will never block the current thread.
    ///
    /// [`Ok`]: std::result::Result::Ok
    /// [`Err`]: std::result::Result::Err
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use channel::channel;
    ///
    /// let (mut tx, mut rx) = channel();
    ///
    /// // This send is always successful.
    /// tx.send(1).unwrap();
    ///
    /// // This send will fail because the receiver is gone
    /// drop(rx);
    /// assert_eq!(tx.send(1).unwrap_err().0, 1);
    /// ```
    pub fn send(&mut self, t: T) -> Result<(), SendError<T>> {
        let mut queue = self.inner.queue.lock().unwrap();
        queue.push_back(t);
        drop(queue);
        self.inner.available.notify_one();
        Ok(())
    }

    pub fn try_send(&self, _t: T) -> Result<(), TrySendError<T>> {
        // self.inner.try_send(t)
        Ok(())
    }
}
