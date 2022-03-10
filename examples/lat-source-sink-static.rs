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

use clap::Parser;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::runtime::dataflow::instance::DataflowInstance;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow::{model::link::PortDescriptor, Node};
use zenoh_flow_perf::operators::{LatSink, LatSource, LAT_PORT};

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u64,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::parse();

    let interval = 1.0 / (args.msgs as f64);

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
        zenoh_flow::runtime::dataflow::Dataflow::new(ctx.clone(), "lat-static".into(), None);

    let source = Arc::new(LatSource {});
    let sink = Arc::new(LatSink {});

    // let operator = Arc::new(NoOp {});

    let config = serde_json::json!({"interval" : interval, "pipeline":args.pipeline, "msgs": args.msgs, "multi":false});
    let config = Some(config);

    zf_graph
        .try_add_static_source(
            "lat-source".into(),
            None,
            PortDescriptor {
                port_id: String::from(LAT_PORT).into(),
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
                port_id: String::from(LAT_PORT).into(),
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
                output: String::from(LAT_PORT).into(),
            },
            InputDescriptor {
                node: format!("lat-sink").into(),
                input: String::from(LAT_PORT).into(),
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
