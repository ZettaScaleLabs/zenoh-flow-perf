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
use zenoh_flow::model::link::{LinkFromDescriptor, LinkToDescriptor};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{
    default_input_rule, default_output_rule, downcast, downcast_mut, model::link::PortDescriptor,
    zf_empty_state, Context, Data, Node, NodeOutput, Operator, PortId, Sink, Source, ZFError,
    ZFResult, ZFState,
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
    pub data: Vec<u8>,
}

#[async_trait]
impl Source for ThrSource {
    async fn run(
        &self,
        _context: &mut Context,
        state: &mut Box<dyn zenoh_flow::ZFState>,
    ) -> zenoh_flow::ZFResult<Data> {
        let real_state = downcast!(ThrSourceState, state).unwrap();

        let data = ThrData {
            data: real_state.data.clone(),
        };

        Ok(Data::from::<ThrData>(data))
    }
}

impl Node for ThrSource {
    fn initialize(
        &self,
        configuration: &Option<HashMap<String, String>>,
    ) -> Box<dyn zenoh_flow::ZFState> {
        let payload_size = match configuration {
            Some(conf) => conf.get("payload_size").unwrap().parse::<usize>().unwrap(),
            None => 8usize,
        };

        let data = (0usize..payload_size)
            .map(|i| (i % 10) as u8)
            .collect::<Vec<u8>>();

        Box::new(ThrSourceState { data })
    }

    fn clean(&self, _state: &mut Box<dyn ZFState>) -> ZFResult<()> {
        Ok(())
    }
}

// SINK

struct ThrSink;

#[derive(ZFState, Debug, Clone)]
struct SinkState {
    pub payload_size: usize,
    pub accumulator: Arc<AtomicUsize>,
    pub abort_handle: AbortHandle,
}

#[async_trait]
impl Sink for ThrSink {
    async fn run(
        &self,
        _context: &mut Context,
        state: &mut Box<dyn zenoh_flow::ZFState>,
        _input: zenoh_flow::runtime::message::DataMessage,
    ) -> zenoh_flow::ZFResult<()> {
        let my_state = downcast!(SinkState, state).unwrap();
        my_state.accumulator.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl Node for ThrSink {
    fn initialize(
        &self,
        configuration: &Option<HashMap<String, String>>,
    ) -> Box<dyn zenoh_flow::ZFState> {
        let payload_size = match configuration {
            Some(conf) => conf.get("payload_size").unwrap().parse::<usize>().unwrap(),
            None => 8usize,
        };

        let accumulator = Arc::new(AtomicUsize::new(0usize));

        let loop_accumulator = Arc::clone(&accumulator);
        let loop_payload_size = payload_size.clone();

        let print_loop = async move {
            println!("layer,scenario,test,name,size,messages");
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

        Box::new(SinkState {
            payload_size,
            accumulator,
            abort_handle,
        })
    }

    fn clean(&self, _state: &mut Box<dyn ZFState>) -> ZFResult<()> {
        let real_state = downcast_mut!(SinkState, _state).unwrap();

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
        state: &mut Box<dyn zenoh_flow::ZFState>,
        tokens: &mut HashMap<PortId, zenoh_flow::Token>,
    ) -> zenoh_flow::ZFResult<bool> {
        default_input_rule(state, tokens)
    }

    fn run(
        &self,
        _context: &mut zenoh_flow::Context,
        _state: &mut Box<dyn zenoh_flow::ZFState>,
        inputs: &mut HashMap<PortId, zenoh_flow::runtime::message::DataMessage>,
    ) -> zenoh_flow::ZFResult<HashMap<zenoh_flow::PortId, Data>> {
        let mut results: HashMap<PortId, Data> = HashMap::new();

        let data = inputs
            .get_mut(PORT)
            .ok_or_else(|| ZFError::InvalidData("No data".to_string()))?
            .data
            .try_get::<ThrData>()?;

        results.insert(PORT.into(), Data::from::<ThrData>(data.clone()));
        Ok(results)
    }

    fn output_rule(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut Box<dyn zenoh_flow::ZFState>,
        outputs: HashMap<PortId, Data>,
    ) -> zenoh_flow::ZFResult<HashMap<zenoh_flow::PortId, NodeOutput>> {
        default_output_rule(state, outputs)
    }
}

impl Node for NoOp {
    fn initialize(
        &self,
        _configuration: &Option<HashMap<String, String>>,
    ) -> Box<dyn zenoh_flow::ZFState> {
        zf_empty_state!()
    }

    fn clean(&self, _state: &mut Box<dyn ZFState>) -> ZFResult<()> {
        Ok(())
    }
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::from_args();

    let session =
        async_std::sync::Arc::new(zenoh::net::open(zenoh::net::config::peer()).await.unwrap());
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());
    let rt_uuid = uuid::Uuid::new_v4();
    let ctx = RuntimeContext {
        session,
        hlc,
        runtime_name: format!("thr-runtime-{}", rt_uuid).into(),
        runtime_uuid: rt_uuid,
    };

    let mut zf_graph = zenoh_flow::runtime::graph::DataFlowGraph::new(ctx.clone());

    let source = Arc::new(ThrSource {});
    let sink = Arc::new(ThrSink {});
    let operator = Arc::new(NoOp {});

    let mut config = HashMap::new();
    config.insert("payload_size".to_string(), format!("{}", args.size));

    zf_graph
        .add_static_source(
            "thr-source".into(),
            PortDescriptor {
                port_id: String::from(PORT),
                port_type: String::from("thr"),
            },
            source,
            Some(config.clone()),
        )
        .unwrap();

    zf_graph
        .add_static_sink(
            "thr-sink".into(),
            PortDescriptor {
                port_id: String::from(PORT),
                port_type: String::from("thr"),
            },
            sink,
            Some(config),
        )
        .unwrap();

    zf_graph
        .add_static_operator(
            "noop".into(),
            vec![PortDescriptor {
                port_id: String::from(PORT),
                port_type: String::from("thr"),
            }],
            vec![PortDescriptor {
                port_id: String::from(PORT),
                port_type: String::from("thr"),
            }],
            operator,
            None,
        )
        .unwrap();

    zf_graph
        .add_link(
            LinkFromDescriptor {
                node: "thr-source".into(),
                output: String::from(PORT),
            },
            LinkToDescriptor {
                node: "noop".into(),
                input: String::from(PORT),
            },
            None,
            None,
            None,
        )
        .unwrap();

    zf_graph
        .add_link(
            LinkFromDescriptor {
                node: "noop".into(),
                output: String::from(PORT),
            },
            LinkToDescriptor {
                node: "thr-sink".into(),
                input: String::from(PORT),
            },
            None,
            None,
            None,
        )
        .unwrap();

    zf_graph.make_connections().await.unwrap();

    let mut managers = vec![];

    let runners = zf_graph.get_runners();
    for runner in &runners {
        let m = runner.start();
        managers.push(m)
    }

    zenoh_flow::async_std::task::sleep(std::time::Duration::from_secs(args.duration)).await;

    for m in managers.iter() {
        m.kill().await.unwrap()
    }

    futures::future::join_all(managers).await;
}
