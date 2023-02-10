use channels::channel;

#[test]
fn test_sender() {
  let (mut sender, receiver) = channel();
  let mut sender2 = sender.clone();

  // First thread owns `sender`.
  std::thread::spawn(move || {
    sender.send(1).unwrap();
  });

  // Second thread owns `sender2`.
  std::thread::spawn(move || {
    sender2.send(2).unwrap();
  });

  let msg = receiver.recv().unwrap();
  let msg2 = receiver.recv().unwrap();

  assert_eq!(msg + msg2, 3);
}

#[test]
fn test_send() {
  let (mut tx, _) = channel();

  // This send is always successful.
  assert!(tx.send(1).is_ok());
}
