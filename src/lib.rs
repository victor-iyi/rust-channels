//! Muli-producer, single consumer FIFO queue communication primitives.
//!
//! This module provides message-based communication over channels, concretely
//! defined among three types.
//!
//! - [`Sender`]
//! - [`SyncSender]
//! - [`Receiver`]
//!
//! A [`Sender`] or [`SyncSender`] is used to send data to a [`Receiver`]. Both
//! senders are clone-able (multi-producer) such that many threads can send
//! simultaneously to one receiver (single-consumer).

use std::sync::Arc;

mod error;
mod inner;
mod iterators;
mod receiver;
mod sender;

use inner::Inner;

// Re-exports.
pub use receiver::Receiver;
pub use sender::Sender;

/// Creates a new asynchronous channel, returning the sender/receiver halves.
/// All data set on the [`Sender`] will become available on the [`Receiver`] in
/// the same order as it was sent, and no [`send`] will block the calling thread
/// (this channel has an "infinite buffer", unlike [`sync_channel`], which will
/// block after its buffer limit is reached). [`recv`] will block until a message
/// is available.
///
/// The [`Sender`] can be cloned to [`send`] to the same channel multiple times,
/// but only one [`Receiver`] is supported.
///
/// If the [`Receiver`] is disconnected while trying to [`send`] with the [`Sender`],
/// the [`send`] method will return a [`SendError`]. Similarly, if the [`Sender`]
/// is disconnected while trying to [`recv`], the [`recv`] method will return a
/// [`RecvError`].
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
  let inner = Arc::new(Inner::new());
  (Sender::new(inner.clone()), Receiver::new(inner))
}
