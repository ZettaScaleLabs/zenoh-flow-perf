use clap::Parser;
use erdos::dataflow::context::*;
use erdos::dataflow::operator::*;
use erdos::dataflow::stream::*;
use erdos::dataflow::time::Timestamp;
use erdos::dataflow::Message;
use erdos::Configuration;
use serde::{Deserialize, Serialize};

use erdos::node::Node;
use std::time::{SystemTime, UNIX_EPOCH};

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u64,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Latency {
    pub ts: u128,
}

pub fn get_epoch_us() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

#[derive(Clone)]
pub struct LatSource {
    interval: f64,
}

impl LatSource {
    pub fn new(interval: f64) -> Self {
        Self { interval }
    }
}

impl Source<Latency> for LatSource {
    fn run(&mut self, _config: &OperatorConfig, write_stream: &mut WriteStream<Latency>) {
        let mut count = 0u64;
        // let interval : f64 = config.name.as_ref().unwrap().parse().unwrap();
        loop {
            let data = Latency { ts: get_epoch_us() };

            write_stream
                .send(Message::new_message(
                    Timestamp::Time(vec![count as u64]),
                    data,
                ))
                .unwrap();
            std::thread::sleep(std::time::Duration::from_secs_f64(self.interval));
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

impl OneInOneOut<(), Latency, Latency> for NoOp {
    fn setup(&mut self, _ctx: &mut SetupContext<()>) {
        ()
    }

    fn on_data(&mut self, ctx: &mut OneInOneOutContext<(), Latency>, data: &Latency) {
        let timestamp = ctx.timestamp().clone();
        log::trace!("Received {:?}", data);
        ctx.write_stream()
            .send(Message::new_message(timestamp, data.clone()))
            .unwrap();
    }

    fn on_watermark(&mut self, _ctx: &mut OneInOneOutContext<'_, (), Latency>) {}

    fn run(
        &mut self,
        _config: &OperatorConfig,
        _read_stream: &mut ReadStream<Latency>,
        _write_stream: &mut WriteStream<Latency>,
    ) {
    }
    fn destroy(&mut self) {}
}

#[derive(Clone)]
pub struct LatSink {
    pub msgs: u64,
    pub size: u64,
}

impl LatSink {
    pub fn new(msgs: u64, size: u64) -> Self {
        Self { msgs, size }
    }
}

impl Sink<(), Latency> for LatSink {
    fn on_data(&mut self, _ctx: &mut SinkContext<'_, ()>, data: &Latency) {
        let now = get_epoch_us();
        let elapsed = now - data.ts;

        println!("erdos,single,latency,1,8,{},{elapsed},us", self.msgs);
    }
    fn on_watermark(&mut self, _ctx: &mut SinkContext<'_, ()>) {}

    fn setup(&mut self, _setup_context: &mut SetupContext<()>) {}
    fn run(&mut self, _config: &OperatorConfig, _read_stream: &mut ReadStream<Latency>) {}
    fn destroy(&mut self) {}
}

fn main() {
    env_logger::init();
    let args = CallArgs::parse();
    let cpus = num_cpus::get();
    let interval = 1.0 / args.msgs as f64;

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
        move || LatSource::new(interval.clone()),
        OperatorConfig::new().name("LatSource"),
    );

    let output_stream = erdos::connect_one_in_one_out(
        NoOp::new,
        || {},
        OperatorConfig::new().name("NoOp"),
        &source_stream,
    );

    erdos::connect_sink(
        move || LatSink::new(args.msgs, 8),
        || {},
        OperatorConfig::new().name("LatSink"),
        &output_stream,
    );

    node.run();
}
