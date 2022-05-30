use clap::Parser;
use erdos::dataflow::context::*;
use erdos::dataflow::operator::*;
use erdos::dataflow::stream::*;
use erdos::dataflow::time::Timestamp;
use erdos::dataflow::Message;
use erdos::node::Node;
use erdos::Configuration;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

static DEFAULT_SIZE: &str = "8";
static DEFAULT_DURATION: &str = "60";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_SIZE)]
    size: usize,
    #[clap(short, long, default_value = DEFAULT_DURATION)]
    duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrData {
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct ThrSource {
    data: ThrData,
}

impl ThrSource {
    pub fn new(size: usize) -> Self {
        let data = ThrData {
            data: (0usize..size).map(|i| (i % 10) as u8).collect::<Vec<u8>>(),
        };
        Self { data }
    }
}

impl Source<ThrData> for ThrSource {
    fn run(&mut self, _config: &OperatorConfig, write_stream: &mut WriteStream<ThrData>) {
        let mut count = 0u64;
        loop {
            let data = self.data.clone();

            write_stream
                .send(Message::new_message(
                    Timestamp::Time(vec![count as u64]),
                    data,
                ))
                .unwrap();

            count = count.wrapping_add(1);
        }
    }
    fn destroy(&mut self) {}
}

#[derive(Clone)]
pub struct NoOp {}

impl NoOp {
    pub fn new() -> Self {
        Self {}
    }
}

impl OneInOneOut<(), ThrData, ThrData> for NoOp {
    fn setup(&mut self, _ctx: &mut SetupContext<()>) {
        ()
    }

    fn on_data(&mut self, ctx: &mut OneInOneOutContext<(), ThrData>, data: &ThrData) {
        let timestamp = ctx.timestamp().clone();
        ctx.write_stream()
            .send(Message::new_message(timestamp, data.clone()))
            .unwrap();
    }

    fn on_watermark(&mut self, _ctx: &mut OneInOneOutContext<'_, (), ThrData>) {}

    fn run(
        &mut self,
        _config: &OperatorConfig,
        _read_stream: &mut ReadStream<ThrData>,
        _write_stream: &mut WriteStream<ThrData>,
    ) {
    }
    fn destroy(&mut self) {}
}

#[derive(Clone)]
pub struct ThrSink {
    pub size: usize,
    pub accumulator: Arc<AtomicUsize>,
}

impl ThrSink {
    pub fn new(size: usize) -> Self {
        let accumulator = Arc::new(AtomicUsize::new(0));

        let loop_accumulator = accumulator.clone();
        let loop_payload_size = size.clone();

        let print_loop = move || {
            loop {
                let now = Instant::now();
                std::thread::sleep(std::time::Duration::from_secs(1));
                let elapsed = now.elapsed().as_micros() as f64;

                let c = loop_accumulator.swap(0, Ordering::Relaxed);
                if c > 0 {
                    let interval = 1_000_000.0 / elapsed;
                    let msgs = (c as f64 / interval).floor() as usize;
                    // framework, scenario, test, pipeline, payload, rate, value, unit
                    println!("erdos,single,throughput,1,{loop_payload_size},{msgs},{msgs},msgs");
                }
            }
        };

        std::thread::spawn(print_loop);

        Self { size, accumulator }
    }
}

impl Sink<(), ThrData> for ThrSink {
    fn on_data(&mut self, _ctx: &mut SinkContext<'_, ()>, _data: &ThrData) {
        self.accumulator.fetch_add(1, Ordering::Relaxed);
    }
    fn on_watermark(&mut self, _ctx: &mut SinkContext<'_, ()>) {}

    fn setup(&mut self, _setup_context: &mut SetupContext<()>) {}
    fn run(&mut self, _config: &OperatorConfig, _read_stream: &mut ReadStream<ThrData>) {}
    fn destroy(&mut self) {}
}

fn main() {
    env_logger::init();
    let args = CallArgs::parse();
    let cpus = num_cpus::get();

    log::debug!("Arguments are {:?}", args);

    let erdos_config = Configuration {
        index: 0,
        num_threads: cpus,
        data_addresses: vec!["127.0.0.1:9900".parse().unwrap()],
        control_addresses: vec!["127.0.0.1:9901".parse().unwrap()],
        graph_filename: None,
        logging_level: None,
    };

    let mut node = Node::new(erdos_config);

    let source_stream = erdos::connect_source(
        move || ThrSource::new(args.size.clone()),
        OperatorConfig::new().name("LatSource"),
    );

    let output_stream = erdos::connect_one_in_one_out(
        NoOp::new,
        || {},
        OperatorConfig::new().name("NoOp"),
        &source_stream,
    );

    erdos::connect_sink(
        move || ThrSink::new(args.size.clone()),
        || {},
        OperatorConfig::new().name("LatSink"),
        &output_stream,
    );

    node.run();
}
