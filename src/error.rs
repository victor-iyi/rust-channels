use std::fmt;

/// The errror returned from the [`Sender::send`] or [`SyncSender::send`]
/// function on **channels**.
///
/// A **send** operation can only fail if the receiving end of a channel is
/// disconnected, implying that the data could never be received. The error
/// contains the data being sent as a payload so it can be recovered.
///
/// [`Sender::send`]: struct.Sender#method.send
/// [`SyncSender::send`]: struct.SyncSender#method.send
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(pub T);

impl<T: fmt::Debug> fmt::Debug for SendError<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_fmt(format_args!("SendError {:?}", self.0))
  }
}

impl<T: fmt::Display> fmt::Display for SendError<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("sending on a closed channel")
  }
}

/// This enumeration is the list of the possible error outcomes for the
/// [`try_send`] method.
///
/// [`try_send`]: SyncSender::try_send
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T> {
  /// The data could not be sent on the [`sync_channel`] because it would require
  /// that the callee block to send the data.
  ///
  /// If this is a buffered channel, then the buffer is full at this time. If
  /// this is not a buffered channel, then there is no [`Receiver`] available
  /// to acquire the data.
  Full(T),
  /// This [`sync_channel`]'s receiving half has disconnected, so the data could
  /// not be sent. The data is returned back to the callee in this case.
  Disconnected(T),
}

impl<T> fmt::Debug for TrySendError<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      TrySendError::Full(..) => "Full(..)".fmt(f),
      TrySendError::Disconnected(..) => "Disconnected(..)".fmt(f),
    }
  }
}

impl<T> fmt::Display for TrySendError<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      TrySendError::Full(..) => "sending on a full channel".fmt(f),
      TrySendError::Disconnected(..) => "sending on a closed channel".fmt(f),
    }
  }
}

impl<T> From<SendError<T>> for TrySendError<T> {
  /// Converts a `SendError<T>` into a `TrySendError<T>`.
  ///
  /// This conversion always returns a `TrySendError::Disconnected` containing
  /// the data in the `SendError<T>`.
  ///
  /// No data is allocated on the heap.
  fn from(err: SendError<T>) -> TrySendError<T> {
    match err {
      SendError(t) => TrySendError::Disconnected(t),
    }
  }
}

/// An error returned from the [`recv`] function on a [`Receiver`].
///
/// The [`recv`] operation can only fail if the sending half of a [`channel`]
/// (or [`sync_channel`]) is disconnected, implying that no further messages
/// will be received.
///
/// [`recv`]: Receiver::recv
/// [`Receiver`]: struct.Receiver
/// [`channel`]: fn.channel
/// [`sync_channel`]: fn.sync_channel
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct RecvError;

impl fmt::Debug for RecvError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RecvError").finish()
  }
}

impl fmt::Display for RecvError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("receiving on a closed channel")
  }
}

/// This enumeration is the list of the possible reasons that [`try_recv`] could
/// not return data when called. This can occur with both a [`channel`] and
/// a [`sync_channel`].
///
/// [`try_recv`]: Receiver::try_recv
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TryRecvError {
  /// This **channel** is currently empty, but the **Sender**(s) have not yet
  /// disconnected, so data may yet become available.
  Empty,
  /// The **channel**'s sending half has become disconnected, and there will
  /// never be any more data received on it.
  Disconnected,
}

impl fmt::Display for TryRecvError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      TryRecvError::Empty => "receiving on an empty channel".fmt(f),
      TryRecvError::Disconnected => "receiving on a closed channel".fmt(f),
    }
  }
}

impl From<RecvError> for TryRecvError {
  /// Converts a `RecvError` into a `TryRecvError`.
  ///
  /// This conversion always returns `TryRecvError::Disconnected`.
  ///
  /// No data is allocated on the heap.
  fn from(err: RecvError) -> TryRecvError {
    match err {
      RecvError => TryRecvError::Disconnected,
    }
  }
}

/// This enumeration is the list of possible errors that made [`recv_timeout`]
/// unable to return data when called. This can occur with both a [`channel`]
/// and a [`sync_channel`].
///
/// [`recv_timeout`]: Reciever::recv_timeout
pub enum RecvTimeoutError {
  /// This **channel** is currently empty, by the **Sender**(s) have not yet
  /// disconnected, so data may yet become available.
  Timeout,
  /// The **channel**'s sending half has become disconnected, and there will
  /// never be any more data received on it.
  Disconnected,
}

impl From<RecvError> for RecvTimeoutError {
  /// Converts a `RecvError` into a `RecvTimeoutError`.
  ///
  /// This conversion always returns `RecvTimeoutError::Disconnected`.
  ///
  /// No data is allocated on the heap.
  fn from(err: RecvError) -> RecvTimeoutError {
    match err {
      RecvError => RecvTimeoutError::Disconnected,
    }
  }
}
