//! Implement 1-of-2 oblivious transfer trait

trait OTSender<N> {
    fn send(values: [N; 2]);
}

trait OTReceiver {
    fn receive();
}
