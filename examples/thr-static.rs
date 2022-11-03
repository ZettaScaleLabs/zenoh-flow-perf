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
use zenoh_flow_perf::nodes::{NoOpFactory, ThrSinkFactory, ThrSourceFactory, THR_PORT};

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

    let session = Arc::new(
        zenoh::open(zenoh::config::Config::default())
            .res()
            .await
            .unwrap(),
    );
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());
    let rt_uuid = uuid::Uuid::new_v4();
    let runtime_name: Arc<str> = format!("thr-static-runtime-{}", rt_uuid).into();
    let ctx = RuntimeContext {
        session,
        hlc,
        loader: Arc::new(Loader::new(LoaderConfig::new())),
        runtime_name: runtime_name.clone(),
        runtime_uuid: rt_uuid,
    };

    let config = serde_json::json!({"payload_size" : args.size, "multi": false});
    let config = Some(config);

    let mut zf_graph = zenoh_flow::runtime::dataflow::DataFlow::new("thr-static", ctx.clone());

    /*
     * Source
     */
    let source_record = SourceRecord {
        id: "thr-source".into(),
        uid: 0u32,
        outputs: vec![PortRecord {
            port_id: THR_PORT.into(),
            port_type: "thr".into(),
            uid: 1u32,
        }],
        uri: None,
        configuration: config.clone(),
        runtime: runtime_name.clone(),
    };

    zf_graph.add_source_factory(source_record, Arc::new(ThrSourceFactory));

    /*
     * Sink
     */
    let sink_record = SinkRecord {
        id: "thr-sink".into(),
        uid: 10u32,
        inputs: vec![PortRecord {
            port_id: THR_PORT.into(),
            port_type: "thr".into(),
            uid: 11u32,
        }],
        uri: None,
        configuration: config.clone(),
        runtime: runtime_name.clone(),
    };

    zf_graph.add_sink_factory(sink_record, Arc::new(ThrSinkFactory));

    /*
     * Operators
     */
    let operator_record = OperatorRecord {
        id: "noop".into(),
        uid: 20,
        inputs: vec![PortRecord {
            port_id: THR_PORT.into(),
            port_type: "thr".into(),
            uid: 21,
        }],
        outputs: vec![PortRecord {
            port_id: THR_PORT.into(),
            port_type: "thr".into(),
            uid: 22,
        }],
        uri: None,
        configuration: config.clone(),
        runtime: runtime_name.clone(),
    };

    zf_graph.add_operator_factory(operator_record, Arc::new(NoOpFactory));

    /*
     * Links
     */
    zf_graph.add_link(
        OutputDescriptor {
            node: "thr-source".into(),
            output: String::from(THR_PORT).into(),
        },
        InputDescriptor {
            node: "noop".into(),
            input: String::from(THR_PORT).into(),
        },
    );

    zf_graph.add_link(
        OutputDescriptor {
            node: "noop".into(),
            output: String::from(THR_PORT).into(),
        },
        InputDescriptor {
            node: "thr-sink".into(),
            input: String::from(THR_PORT).into(),
        },
    );

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

    async_std::task::sleep(std::time::Duration::from_secs(args.duration)).await;
}
