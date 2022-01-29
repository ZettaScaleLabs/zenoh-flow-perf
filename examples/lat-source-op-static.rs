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
use std::collections::HashMap;
use std::time::Duration;
use structopt::StructOpt;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::runtime::dataflow::instance::DataflowInstance;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{
    default_input_rule, default_output_rule, model::link::PortDescriptor, Configuration, Context,
    Data, LocalDeadlineMiss, Node, NodeOutput, Operator, PortId, Source, State, ZFResult,
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

// OPERATOR

#[derive(ZFState, Debug, Clone)]
struct LatOpState {
    pipeline: u64,
    interval: f64,
    msgs: u64,
}

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
        state: &mut State,
        inputs: &mut HashMap<PortId, zenoh_flow::runtime::message::DataMessage>,
    ) -> zenoh_flow::ZFResult<HashMap<zenoh_flow::PortId, Data>> {
        let results: HashMap<PortId, Data> = HashMap::new();

        let real_state = state.try_get::<LatOpState>()?;
        let _ = real_state.interval;

        let data = inputs
            .get_mut(PORT)
            .unwrap()
            .get_inner_data()
            .try_get::<Latency>()?;

        let now = get_epoch_us();

        let elapsed = now - data.ts;
        let msgs = real_state.msgs;
        let pipeline = real_state.pipeline;
        // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
        println!("zf-source-op,scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");

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

        Ok(State::from(LatOpState {
            interval,
            pipeline,
            msgs,
        }))
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
    let op = Arc::new(NoOp {});

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
        .try_add_static_operator(
            format!("noop").into(),
            vec![PortDescriptor {
                port_id: String::from(PORT).into(),
                port_type: String::from("lat").into(),
            }],
            vec![],
            None,
            op.initialize(&config).unwrap(),
            op,
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
                node: format!("noop").into(),
                input: String::from(PORT).into(),
            },
            None,
            None,
            None,
        )
        .unwrap();
    pipe.push_str(format!("lat-source-->noop").as_str());

    // println!("Pipeline is: {pipe}");

    let mut instance = DataflowInstance::try_instantiate(zf_graph).unwrap();

    let nodes = instance.get_nodes();
    for id in &nodes {
        instance.start_node(id).await.unwrap()
    }

    let () = std::future::pending().await;
}
