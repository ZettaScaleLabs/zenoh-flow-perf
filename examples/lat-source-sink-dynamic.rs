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

use std::convert::TryFrom;
use std::fs::{File, *};
use std::io::Write;
use std::path::Path;
use structopt::StructOpt;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::model::dataflow::descriptor::{DataFlowDescriptor, Mapping};
use zenoh_flow::model::link::{LinkDescriptor, PortDescriptor};
use zenoh_flow::model::node::{SinkDescriptor, SourceDescriptor};
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";
static DEFAULT_RT_NAME: &str = "nothing";
static DEFAULT_RT_DESCRIPTOR: &str = "./descriptor.yaml";

static SRC_URI: &str = "file://./target/release/examples/libdyn_source.so";
static SNK_URI: &str = "file://./target/release/examples/libdyn_sink.so";

static PORT: &str = "Data";

#[derive(StructOpt, Debug)]
struct CallArgs {
    #[structopt(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u64,
    #[structopt(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
    #[structopt(short, long)]
    runtime: bool,
    #[structopt(short, long, default_value = DEFAULT_RT_NAME)]
    name: String,
    #[structopt(short, long, default_value = DEFAULT_RT_DESCRIPTOR)]
    descriptor_file: String,
}

fn write_string_to_file(content: String, filename: &str) {
    let path = Path::new(filename);
    let mut write_file = File::create(path).unwrap();
    write!(write_file, "{content}").unwrap();
}

async fn runtime(name: String, descriptor_file: String) {
    env_logger::init();

    let yaml_df = read_to_string(descriptor_file).unwrap();

    let loader_config = LoaderConfig::new();

    let session = Arc::new(zenoh::open(zenoh::config::Config::default()).await.unwrap());
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());
    let loader = Arc::new(Loader::new(loader_config));

    let ctx = RuntimeContext {
        session,
        hlc,
        loader,
        runtime_name: name.clone().into(),
        runtime_uuid: uuid::Uuid::new_v4(),
    };

    // loading the descriptor
    let df =
        zenoh_flow::model::dataflow::descriptor::DataFlowDescriptor::from_yaml(&yaml_df).unwrap();

    // mapping to infrastructure
    let mapped = zenoh_flow::runtime::map_to_infrastructure(df, &name)
        .await
        .unwrap();

    // creating record
    let dfr =
        zenoh_flow::model::dataflow::record::DataFlowRecord::try_from((mapped, uuid::Uuid::nil()))
            .unwrap();

    // creating dataflow
    let dataflow = zenoh_flow::runtime::dataflow::Dataflow::try_new(ctx.clone(), dfr).unwrap();

    // instantiating
    let mut instance =
        zenoh_flow::runtime::dataflow::instance::DataflowInstance::try_instantiate(dataflow)
            .unwrap();

    let mut sinks = instance.get_sinks();
    for id in sinks.drain(..) {
        instance.start_node(&id).await.unwrap()
    }

    let mut operators = instance.get_operators();
    for id in operators.drain(..) {
        instance.start_node(&id).await.unwrap()
    }

    let mut connectors = instance.get_connectors();
    for id in connectors.drain(..) {
        instance.start_node(&id).await.unwrap()
    }

    let sources = instance.get_sources();
    for id in &sources {
        instance.start_node(id).await.unwrap()
    }

    let () = std::future::pending().await;
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    let args = CallArgs::from_args();

    if args.runtime {
        runtime(args.name, args.descriptor_file.clone()).await;
    }

    let interval = 1.0 / (args.msgs as f64);

    let config = Some(
        serde_json::json!({"interval" : interval, "pipeline":args.pipeline, "msgs": args.msgs, "multi":true}),
    );

    // Creating the descriptor

    let mut dfd = DataFlowDescriptor {
        flow: format!("pipeline-source-sink"),
        operators: vec![],
        sources: vec![],
        sinks: vec![],
        links: vec![],
        mapping: None,
        deadlines: None,
        loops: None,
    };

    // Source

    let source_descriptor = SourceDescriptor {
        id: "source".into(),
        period: None,
        output: PortDescriptor {
            port_id: PORT.into(),
            port_type: "latency".into(),
        },
        uri: Some(String::from(SRC_URI)),
        configuration: config.clone(),
        runtime: None,
    };

    // Sink

    let sink_descriptor = SinkDescriptor {
        id: "sink".into(),
        input: PortDescriptor {
            port_id: PORT.into(),
            port_type: "latency".into(),
        },
        uri: Some(String::from(SNK_URI)),
        configuration: config.clone(),
        runtime: None,
    };

    // Adding source and sinks to descriptor
    dfd.sources.push(source_descriptor);
    dfd.sinks.push(sink_descriptor);

    // Creating and adding source to first operator link

    let source_sink_link = LinkDescriptor {
        from: OutputDescriptor {
            node: "source".into(),
            output: PORT.into(),
        },
        to: InputDescriptor {
            node: format!("sink").into(),
            input: PORT.into(),
        },
        size: None,
        queueing_policy: None,
        priority: None,
    };

    dfd.links.push(source_sink_link);

    // Creating static mapping

    // Source mapping
    dfd.add_mapping(Mapping {
        id: "source".into(),
        runtime: "src".into(),
    });

    // Sink mapping
    dfd.add_mapping(Mapping {
        id: "sink".into(),
        runtime: "snk".into(),
    });

    let yaml_descriptor = dfd.to_yaml().unwrap();
    println!("Descriptor:\n{yaml_descriptor}");

    write_string_to_file(yaml_descriptor, &args.descriptor_file);

    //let () = std::future::pending().await;
}
