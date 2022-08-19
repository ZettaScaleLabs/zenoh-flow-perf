//
// Copyright (c) 2017, 2022 ZettaScale Technology.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale zenoh team, <zenoh@zettascale.tech>
//

use clap::Parser;
use std::sync::Arc;
use zenoh_flow::model::link::PortDescriptor;
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::runtime::dataflow::instance::DataflowInstance;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow_perf::nodes::{ThrNoOp, ThrSink, ThrSource, THR_PORT};

static DEFAULT_SIZE: &str = "8";
static DEFAULT_DURATION: &str = "60";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_SIZE)]
    size: u64,
    #[clap(short, long, default_value = DEFAULT_DURATION)]
    duration: u64,
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::parse();

    let session = Arc::new(zenoh::open(zenoh::config::Config::default()).await.unwrap());
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());
    let rt_uuid = uuid::Uuid::new_v4();
    let ctx = RuntimeContext {
        session,
        hlc,
        loader: Arc::new(Loader::new(LoaderConfig::new())),
        runtime_name: format!("thr-runtime-{}", rt_uuid).into(),
        runtime_uuid: rt_uuid,
    };

    let mut zf_graph =
        zenoh_flow::runtime::dataflow::Dataflow::new(ctx.clone(), "thr-static".into(), None);

    let source = Arc::new(ThrSource {});
    let sink = Arc::new(ThrSink {});
    let operator = Arc::new(ThrNoOp {});

    let config = serde_json::json!({"payload_size" : args.size, "multi": false});
    let config = Some(config);

    zf_graph
        .try_add_static_source(
            "thr-source".into(),
            config.clone(),
            vec![PortDescriptor {
                port_id: String::from(THR_PORT).into(),
                port_type: String::from("thr").into(),
            }],
            source,
        )
        .unwrap();

    zf_graph
        .try_add_static_sink(
            "thr-sink".into(),
            config.clone(),
            vec![PortDescriptor {
                port_id: String::from(THR_PORT).into(),
                port_type: String::from("thr").into(),
            }],
            sink,
        )
        .unwrap();

    zf_graph
        .try_add_static_operator(
            "noop".into(),
            config.clone(),
            vec![PortDescriptor {
                port_id: String::from(THR_PORT).into(),
                port_type: String::from("thr").into(),
            }],
            vec![PortDescriptor {
                port_id: String::from(THR_PORT).into(),
                port_type: String::from("thr").into(),
            }],
            operator,
        )
        .unwrap();

    zf_graph
        .try_add_link(
            OutputDescriptor {
                node: "thr-source".into(),
                output: String::from(THR_PORT).into(),
            },
            InputDescriptor {
                node: "noop".into(),
                input: String::from(THR_PORT).into(),
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
                output: String::from(THR_PORT).into(),
            },
            InputDescriptor {
                node: "thr-sink".into(),
                input: String::from(THR_PORT).into(),
            },
            None,
            None,
            None,
        )
        .unwrap();

    let mut instance = DataflowInstance::try_instantiate(zf_graph, ctx.hlc.clone()).unwrap();

    let nodes = instance.get_nodes();
    for id in &nodes {
        instance.start_node(id).await.unwrap()
    }

    async_std::task::sleep(std::time::Duration::from_secs(args.duration)).await;
}
