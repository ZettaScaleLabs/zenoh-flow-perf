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
use zenoh::prelude::r#async::*;
use zenoh_flow::model::descriptor::{InputDescriptor, OutputDescriptor};
use zenoh_flow::model::record::{OperatorRecord, PortRecord, SinkRecord, SourceRecord};
use zenoh_flow::runtime::dataflow::instance::DataFlowInstance;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow_perf::nodes::{LatSinkFactory, LatSourceFactory, NoOpFactory, LAT_PORT};

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u32,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u32,
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::parse();

    let interval = 1.0 / (args.msgs as f64);

    let session = Arc::new(
        zenoh::open(zenoh::config::Config::default())
            .res()
            .await
            .unwrap(),
    );
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());

    let rt_uuid = uuid::Uuid::new_v4();
    let runtime_name: Arc<str> = format!("thr-runtime-{}", rt_uuid).into();
    let ctx = RuntimeContext {
        session,
        hlc,
        loader: Arc::new(Loader::new(LoaderConfig::new())),
        runtime_name: runtime_name.clone(),
        runtime_uuid: rt_uuid,
    };

    let mut zf_graph = zenoh_flow::runtime::dataflow::DataFlow::new("lat-static", ctx.clone());

    let config = serde_json::json!({"interval" : interval, "pipeline":args.pipeline, "msgs": args.msgs, "multi":false});
    let config = Some(config);

    /*
     * Source
     */
    let source_record = SourceRecord {
        id: "lat-source".into(),
        uid: 0u32,
        outputs: vec![PortRecord {
            port_id: LAT_PORT.into(),
            port_type: "lat".into(),
            uid: 1u32,
        }],
        uri: None,
        configuration: config.clone(),
        runtime: runtime_name.clone(),
    };

    zf_graph.add_source_factory(source_record, Arc::new(LatSourceFactory));

    /*
     * Sink
     */
    let sink_record = SinkRecord {
        id: "lat-sink".into(),
        uid: 10u32,
        inputs: vec![PortRecord {
            port_id: LAT_PORT.into(),
            port_type: "lat".into(),
            uid: 11u32,
        }],
        uri: None,
        configuration: config.clone(),
        runtime: runtime_name.clone(),
    };

    zf_graph.add_sink_factory(sink_record, Arc::new(LatSinkFactory));

    /*
     * Operators
     */
    for i in 0..args.pipeline {
        // We reason by tens, we start at 20 because: 0u32 is for the Source, 10u32 for the Sink
        let uid = args.pipeline * i + 20u32;
        let operator_record = OperatorRecord {
            id: format!("noop{i}").into(),
            uid,
            inputs: vec![PortRecord {
                port_id: LAT_PORT.into(),
                port_type: "lat".into(),
                uid: uid + 1,
            }],
            outputs: vec![PortRecord {
                port_id: LAT_PORT.into(),
                port_type: "lat".into(),
                uid: uid + 2,
            }],
            uri: None,
            configuration: config.clone(),
            runtime: runtime_name.clone(),
        };

        zf_graph.add_operator_factory(operator_record, Arc::new(NoOpFactory));
    }

    /*
     * Links
     */
    let mut pipe = String::from("");
    zf_graph.add_link(
        OutputDescriptor {
            node: "lat-source".into(),
            output: String::from(LAT_PORT).into(),
        },
        InputDescriptor {
            node: "noop0".into(),
            input: String::from(LAT_PORT).into(),
        },
    );
    pipe.push_str("lat-source-->noop0-->");

    for i in 1..args.pipeline {
        // println!("Iteration {i}");

        let j = i - 1;
        zf_graph.add_link(
            OutputDescriptor {
                node: format!("noop{j}").into(),
                output: String::from(LAT_PORT).into(),
            },
            InputDescriptor {
                node: format!("noop{i}").into(),
                input: String::from(LAT_PORT).into(),
            },
        );
        pipe.push_str(format!("noop{j}-->noop{i}-->").as_str());
    }

    let len = args.pipeline - 1;
    zf_graph.add_link(
        OutputDescriptor {
            node: format!("noop{len}").into(),
            output: String::from(LAT_PORT).into(),
        },
        InputDescriptor {
            node: "lat-sink".into(),
            input: String::from(LAT_PORT).into(),
        },
    );
    pipe.push_str(format!("noop{len}-->lat-sink").as_str());

    // println!("Pipeline is: {pipe}");

    /*
     * Creating the instance.
     */
    let mut instance = DataFlowInstance::try_instantiate(zf_graph, ctx.hlc.clone())
        .await
        .unwrap();

    for id in instance.get_sinks() {
        instance.start_node(&id).unwrap();
    }

    for id in instance.get_operators() {
        instance.start_node(&id).unwrap();
    }

    for id in instance.get_connectors() {
        instance.start_node(&id).unwrap();
    }

    for id in instance.get_sources() {
        instance.start_node(&id).unwrap();
    }

    std::future::pending::<()>().await;
}
