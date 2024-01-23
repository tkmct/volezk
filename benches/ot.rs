use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::thread_rng;
use std::{
    io::{BufReader, BufWriter},
    os::unix::net::UnixStream,
    thread,
};
use volezk::{
    block::*,
    ot::co15::{CO15Receiver, CO15Sender},
    ot::extension::iknp::{ot_ext_receive, ot_ext_send},
    Channel,
};

fn iknp() -> Result<(), Box<dyn std::error::Error>> {
    // Do 128 base OT for key exchange
    let (ot_sender_stream, ot_receiver_stream) = UnixStream::pair().unwrap();
    let (ext_sender_stream, ext_receiver_stream) = UnixStream::pair().unwrap();

    let receiver_handle = thread::spawn(move || {
        let mut rng = thread_rng();
        let reader = BufReader::new(ot_sender_stream.try_clone().unwrap());
        let writer = BufWriter::new(ot_sender_stream);
        let sender_channel = Channel::new(reader, writer);
        let mut ot_sender = CO15Sender::setup(sender_channel, &mut rng).unwrap();
        let choices = [
            true, false, true, true, false, true, true, false, true, false,
        ];

        let reader = BufReader::new(ext_receiver_stream.try_clone().unwrap());
        let writer = BufWriter::new(ext_receiver_stream);
        let mut ext_receiver_chan = Channel::new(reader, writer);

        ot_ext_receive::<
            CO15Sender<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>,
            Block128,
            [Block128; 1],
            10,
            Channel<BufReader<UnixStream>, BufWriter<UnixStream>>,
        >(&mut ot_sender, choices, &mut ext_receiver_chan)
    });

    // Prepare sender
    let reader = BufReader::new(ot_receiver_stream.try_clone().unwrap());
    let writer = BufWriter::new(ot_receiver_stream);
    let receiver_channel = Channel::new(reader, writer);
    let mut ot_receiver = CO15Receiver::setup(receiver_channel).unwrap();

    let values = [
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
        [Block128::from(1), Block128::from(100)],
    ];
    let reader = BufReader::new(ext_sender_stream.try_clone().unwrap());
    let writer = BufWriter::new(ext_sender_stream);
    let mut ext_sender_chan = Channel::new(reader, writer);

    ot_ext_send::<
        CO15Receiver<Channel<BufReader<UnixStream>, BufWriter<UnixStream>>>,
        Block128,
        // TODO: fix later
        [Block128; 1],
        10,
        Channel<BufReader<UnixStream>, BufWriter<UnixStream>>,
    >(&mut ot_receiver, values, &mut ext_sender_chan)?;

    let receiver_result = receiver_handle.join().unwrap();
    assert!(receiver_result.is_ok());

    // choice for: true, false, true, true, false, true, true, false, true, false,
    let expected_result = [
        Block128::from(100),
        Block128::from(1),
        Block128::from(100),
        Block128::from(100),
        Block128::from(1),
        Block128::from(100),
        Block128::from(100),
        Block128::from(1),
        Block128::from(100),
        Block128::from(1),
    ];

    assert_eq!(receiver_result.unwrap(), expected_result);

    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("OT Extension IKNP: K=128, M=1000", |b| b.iter(iknp));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
