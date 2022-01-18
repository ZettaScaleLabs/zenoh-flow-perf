//
// Copyright (c) 2017, 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//

use async_trait::async_trait;
use futures::future::{AbortHandle, Abortable};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use structopt::StructOpt;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::model::{OutputDescriptor, InputDescriptor};
use zenoh_flow::runtime::dataflow::instance::DataflowInstance;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::LocalDeadlineMiss;
use zenoh_flow::{
    default_input_rule, default_output_rule, model::link::PortDescriptor, zf_empty_state,
    Configuration, Context, Data, Node, NodeOutput, Operator, PortId, Sink, Source, State,
    ZFResult,
};
use zenoh_flow_perf::ThrData;

static DEFAULT_SIZE: &str = "8";
static DEFAULT_DURATION: &str = "60";
static PORT: &str = "Data";

#[derive(StructOpt, Debug)]
struct CallArgs {
    #[structopt(short, long, default_value = DEFAULT_SIZE)]
    size: u64,
    #[structopt(short, long, default_value = DEFAULT_DURATION)]
    duration: u64,
}

// SOURCE

#[derive(Debug)]
struct ThrSource;

#[derive(Debug, ZFState)]
struct ThrSourceState {
    pub data: Arc<ThrData>,
}

#[async_trait]
impl Source for ThrSource {
    async fn run(&self, _context: &mut Context, state: &mut State) -> zenoh_flow::ZFResult<Data> {
        let real_state = state.try_get::<ThrSourceState>()?;

        let data = real_state.data.clone();

        Ok(Data::from_arc::<ThrData>(data))
    }
}

impl Node for ThrSource {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let data = Arc::new(ThrData {
            data: (0usize..payload_size)
                .map(|i| (i % 10) as u8)
                .collect::<Vec<u8>>(),
        });

        Ok(State::from(ThrSourceState { data }))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

// SINK

struct ThrSink;

#[derive(ZFState, Debug, Clone)]
struct SinkState {
    pub _payload_size: usize,
    pub accumulator: Arc<AtomicUsize>,
    pub abort_handle: AbortHandle,
}

#[async_trait]
impl Sink for ThrSink {
    async fn run(
        &self,
        _context: &mut Context,
        state: &mut State,
        _input: zenoh_flow::runtime::message::DataMessage,
    ) -> zenoh_flow::ZFResult<()> {
        let my_state = state.try_get::<SinkState>()?;
        my_state.accumulator.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl Node for ThrSink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let accumulator = Arc::new(AtomicUsize::new(0usize));

        let loop_accumulator = Arc::clone(&accumulator);
        let loop_payload_size = payload_size.clone();

        let print_loop = async move {
            // println!("layer,scenario,test,name,size,messages");
            loop {
                let now = Instant::now();
                async_std::task::sleep(Duration::from_secs(1)).await;
                let elapsed = now.elapsed().as_micros() as f64;

                let c = loop_accumulator.swap(0, Ordering::Relaxed);
                if c > 0 {
                    let interval = 1_000_000.0 / elapsed;
                    println!(
                        "zenoh-flow-static,same-runtime,throughput,{},{},{}",
                        "test-name",
                        loop_payload_size,
                        (c as f64 / interval).floor() as usize
                    );
                }
            }
        };

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let _print_task = async_std::task::spawn(Abortable::new(print_loop, abort_registration));

        Ok(State::from(SinkState {
            _payload_size: payload_size,
            accumulator,
            abort_handle,
        }))
    }

    fn finalize(&self, state: &mut State) -> ZFResult<()> {
        let real_state = state.try_get::<SinkState>()?;

        real_state.abort_handle.abort();
        Ok(())
    }
}

// OPERATOR

#[derive(Debug)]
struct NoOp;

impl Operator for NoOp {
    fn input_rule(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut State,
        tokens: &mut HashMap<PortId, zenoh_flow::InputToken>,
    ) -> zenoh_flow::ZFResult<bool> {
        default_input_rule(state, tokens)
    }

    fn run(
        &self,
        _context: &mut zenoh_flow::Context,
        _state: &mut State,
        inputs: &mut HashMap<PortId, zenoh_flow::runtime::message::DataMessage>,
    ) -> zenoh_flow::ZFResult<HashMap<zenoh_flow::PortId, Data>> {
        let mut results: HashMap<PortId, Data> = HashMap::new();

        let data = inputs
            .get_mut(PORT)
            .unwrap()
            .get_inner_data().clone();

        results.insert(PORT.into(), data);
        Ok(results)
    }

    fn output_rule(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut State,
        outputs: HashMap<PortId, Data>,
        _deadline_miss: Option<LocalDeadlineMiss>,
    ) -> zenoh_flow::ZFResult<HashMap<zenoh_flow::PortId, NodeOutput>> {
        default_output_rule(state, outputs)
    }
}

impl Node for NoOp {
    fn initialize(&self, _configuration: &Option<Configuration>) -> ZFResult<State> {
        zf_empty_state!()
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::from_args();

    let session = Arc::new(zenoh::open(zenoh::config::Config::default()).await.unwrap());
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());
    let rt_uuid = uuid::Uuid::new_v4();
    let ctx = RuntimeContext {
        session,
        hlc,
        loader: Arc::new(Loader::new(LoaderConfig { extensions: vec![] })),
        runtime_name: format!("thr-runtime-{}", rt_uuid).into(),
        runtime_uuid: rt_uuid,
    };

    let mut zf_graph =
        zenoh_flow::runtime::dataflow::Dataflow::new(ctx.clone(), "thr-static".into(), None);

    let source = Arc::new(ThrSource {});
    let sink = Arc::new(ThrSink {});
    let operator = Arc::new(NoOp {});

    let config = serde_json::json!({"payload_size" : args.size});
    let config = Some(config);

    zf_graph.try_add_static_source(
        "thr-source".into(),
        None,
        PortDescriptor {
            port_id: String::from(PORT).into(),
            port_type: String::from("thr").into(),
        },
        source.initialize(&config).unwrap(),
        source,
    ).unwrap();

    zf_graph.try_add_static_sink(
        "thr-sink".into(),
        PortDescriptor {
            port_id: String::from(PORT).into(),
            port_type: String::from("thr").into(),
        },
        sink.initialize(&config).unwrap(),
        sink,
    ).unwrap();

    zf_graph.try_add_static_operator(
        "noop".into(),
        vec![PortDescriptor {
            port_id: String::from(PORT).into(),
            port_type: String::from("thr").into(),
        }],
        vec![PortDescriptor {
            port_id: String::from(PORT).into(),
            port_type: String::from("thr").into(),
        }],
        None,
        operator.initialize(&None).unwrap(),
        operator,
    ).unwrap();

    zf_graph
        .try_add_link(
            OutputDescriptor {
                node: "thr-source".into(),
                output: String::from(PORT).into(),
            },
            InputDescriptor {
                node: "noop".into(),
                input: String::from(PORT).into(),
            },
            None,
            None,
            None,
        )
        .unwrap();

    zf_graph
        .try_add_link(
            OutputDescriptor {
                node: "noop".into(),
                output: String::from(PORT).into(),
            },
            InputDescriptor {
                node: "thr-sink".into(),
                input: String::from(PORT).into(),
            },
            None,
            None,
            None,
        )
        .unwrap();

        let mut instance = DataflowInstance::try_instantiate(zf_graph).unwrap();

        let nodes = instance.get_nodes();
        for id in &nodes {
            instance.start_node(id).await.unwrap()
        }

    zenoh_flow::async_std::task::sleep(std::time::Duration::from_secs(args.duration)).await;
}
