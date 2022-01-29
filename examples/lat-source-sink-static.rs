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
use std::time::Duration;
use structopt::StructOpt;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::runtime::dataflow::instance::DataflowInstance;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{
    model::link::PortDescriptor, Configuration, Context, Data, Node, Sink, Source, State, ZFResult,
};
use zenoh_flow_perf::{get_epoch_us, Latency};

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";
static PORT: &str = "Data";

#[derive(StructOpt, Debug)]
struct CallArgs {
    #[structopt(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u64,
    #[structopt(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
}

// SOURCE

#[derive(Debug)]
struct LatSource;

#[derive(Debug, ZFState)]
struct LatSourceState {
    interval: f64,
}

#[async_trait]
impl Source for LatSource {
    async fn run(&self, _context: &mut Context, state: &mut State) -> zenoh_flow::ZFResult<Data> {
        let real_state = state.try_get::<LatSourceState>()?;

        async_std::task::sleep(Duration::from_secs_f64(real_state.interval)).await;

        let msg = Latency { ts: get_epoch_us() };

        Ok(Data::from::<Latency>(msg))
    }
}

impl Node for LatSource {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        Ok(State::from(LatSourceState { interval }))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

// SINK

struct LatSink;

#[derive(ZFState, Debug, Clone)]
struct LatSinkState {
    pipeline: u64,
    interval: f64,
    msgs: u64,
}

#[async_trait]
impl Sink for LatSink {
    async fn run(
        &self,
        _context: &mut Context,
        state: &mut State,
        mut input: zenoh_flow::runtime::message::DataMessage,
    ) -> zenoh_flow::ZFResult<()> {
        let real_state = state.try_get::<LatSinkState>()?;
        let _ = real_state.interval;

        let data = input.get_inner_data().try_get::<Latency>()?;

        let now = get_epoch_us();

        let elapsed = now - data.ts;
        let msgs = real_state.msgs;
        let pipeline = real_state.pipeline;
        // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
        println!("zf-source-sink,scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");

        Ok(())
    }
}

impl Node for LatSink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let pipeline = match configuration {
            Some(conf) => conf["pipeline"].as_u64().unwrap(),
            None => 1u64,
        };

        let msgs = match configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1u64,
        };

        Ok(State::from(LatSinkState {
            interval,
            pipeline,
            msgs,
        }))
    }

    fn finalize(&self, state: &mut State) -> ZFResult<()> {
        let _real_state = state.try_get::<LatSinkState>()?;

        Ok(())
    }
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::from_args();

    let interval = 1.0 / (args.msgs as f64);

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
        zenoh_flow::runtime::dataflow::Dataflow::new(ctx.clone(), "lat-static".into(), None);

    let source = Arc::new(LatSource {});
    let sink = Arc::new(LatSink {});

    // let operator = Arc::new(NoOp {});

    let config =
        serde_json::json!({"interval" : interval, "pipeline":args.pipeline, "msgs": args.msgs});
    let config = Some(config);

    zf_graph
        .try_add_static_source(
            "lat-source".into(),
            None,
            PortDescriptor {
                port_id: String::from(PORT).into(),
                port_type: String::from("lat").into(),
            },
            source.initialize(&config).unwrap(),
            source,
        )
        .unwrap();

    zf_graph
        .try_add_static_sink(
            "lat-sink".into(),
            PortDescriptor {
                port_id: String::from(PORT).into(),
                port_type: String::from("lat").into(),
            },
            sink.initialize(&config).unwrap(),
            sink,
        )
        .unwrap();

    let mut pipe = String::from("");
    zf_graph
        .try_add_link(
            OutputDescriptor {
                node: "lat-source".into(),
                output: String::from(PORT).into(),
            },
            InputDescriptor {
                node: format!("lat-sink").into(),
                input: String::from(PORT).into(),
            },
            None,
            None,
            None,
        )
        .unwrap();
    pipe.push_str(format!("lat-source-->lat-sink").as_str());

    // println!("Pipeline is: {pipe}");

    let mut instance = DataflowInstance::try_instantiate(zf_graph).unwrap();

    let nodes = instance.get_nodes();
    for id in &nodes {
        instance.start_node(id).await.unwrap()
    }

    let () = std::future::pending().await;
}
