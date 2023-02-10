use channels::{channel, Receiver};
use std::{thread, time::Duration};

#[test]
fn test_try_recv() {
  let (_, receiver): (_, Receiver<i32>) = channel();

  assert!(receiver.try_recv().is_err());
}

#[test]
fn test_recv() {
  let (mut send, recv) = channel();
  let handle = thread::spawn(move || {
    send.send(1u8).unwrap();
  });

  handle.join().unwrap();

  assert_eq!(Ok(1), recv.recv());
}

// #[test]
// fn test_iter() {
//   let (mut send, recv) = channel();
//
//   thread::spawn(move || {
//     send.send(1).unwrap();
//     send.send(2).unwrap();
//     send.send(3).unwrap();
//   });
//
//   let mut iter = recv.iter();
//   assert_eq!(iter.next(), Some(1));
//   assert_eq!(iter.next(), Some(2));
//   assert_eq!(iter.next(), Some(3));
//   assert_eq!(iter.next(), None);
// }

#[test]
fn test_try_iter() {
  let (mut sender, receiver) = channel();

  // Nothing is in the buffer yet.
  assert!(receiver.try_iter().next().is_none());

  thread::spawn(move || {
    thread::sleep(Duration::from_secs(1));
    sender.send(1).unwrap();
    sender.send(2).unwrap();
    sender.send(3).unwrap();
  });

  // Nothing is in the buffer yet.
  assert!(receiver.try_iter().next().is_none());

  // Block for 2 seconds.
  thread::sleep(Duration::from_secs(2));

  let mut iter = receiver.try_iter();
  assert_eq!(iter.next(), Some(1));
  assert_eq!(iter.next(), Some(2));
  assert_eq!(iter.next(), Some(3));
  assert_eq!(iter.next(), None);
}
