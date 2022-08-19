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

use std::collections::HashMap;

use clap::Parser;
use zenoh_flow::model::dataflow::descriptor::FlattenDataFlowDescriptor;
use zenoh_flow::model::link::{LinkDescriptor, PortDescriptor};
use zenoh_flow::model::node::{SimpleOperatorDescriptor, SinkDescriptor, SourceDescriptor};
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::types::{NodeId, RuntimeId};
use zenoh_flow_perf::runtime::Descriptor;

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";
static DEFAULT_RT_NAME: &str = "nothing";
static DEFAULT_RT_DESCRIPTOR: &str = "./descriptor.yaml";

static NOOP_URI: &str = "file://./target/release/examples/libdyn_noop.so";
static SRC_URI: &str = "file://./target/release/examples/libdyn_source.so";
static SNK_URI: &str = "file://./target/release/examples/libdyn_sink.so";

static PING_SRC_URI: &str = "file://./target/release/examples/libdyn_ping.so";
static PONG_SNK_URI: &str = "file://./target/release/examples/libdyn_pong.so";

static PORT: &str = "Data";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u64,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
    #[clap(short, long)]
    runtime: bool,
    #[clap(short, long, default_value = DEFAULT_RT_NAME)]
    name: String,
    #[clap(short, long, default_value = DEFAULT_RT_DESCRIPTOR)]
    descriptor_file: String,
    #[clap(short, long)]
    ping: bool,
    #[clap(short, long)]
    listen: Vec<String>,
    #[clap(short, long)]
    connect: Vec<String>,
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    let args = CallArgs::parse();

    if args.runtime {
        zenoh_flow_perf::runtime::runtime(
            args.name,
            Descriptor::Flatten(args.descriptor_file.clone()),
            args.listen,
            args.connect,
        )
        .await;
    }

    let interval = 1.0 / (args.msgs as f64);

    let config = Some(
        serde_json::json!({"interval" : interval, "pipeline":args.pipeline, "msgs": args.msgs, "multi":true}),
    );

    // Creating the descriptor

    let mut mapping: HashMap<NodeId, RuntimeId> = HashMap::new();
    let mut dfd = FlattenDataFlowDescriptor {
        flow: format!("pipeline{}", args.pipeline),
        operators: vec![],
        sources: vec![],
        sinks: vec![],
        links: vec![],
        mapping: None,
        global_configuration: None,
        flags: None,
    };

    // Source and Sink

    let (source_descriptor, sink_descriptor) = match args.ping {
        true => {
            let src = SourceDescriptor {
                id: "source".into(),
                // period: None,
                outputs: vec![PortDescriptor {
                    port_id: PORT.into(),
                    port_type: "latency".into(),
                }],
                uri: Some(String::from(PING_SRC_URI)),
                configuration: config.clone(),
                tags: vec![],
            };
            let snk = SinkDescriptor {
                id: "sink".into(),
                inputs: vec![PortDescriptor {
                    port_id: PORT.into(),
                    port_type: "latency".into(),
                }],
                uri: Some(String::from(PONG_SNK_URI)),
                configuration: config,
                tags: vec![],
            };
            (src, snk)
        }
        false => {
            let src = SourceDescriptor {
                id: "source".into(),
                // period: None,
                outputs: vec![PortDescriptor {
                    port_id: PORT.into(),
                    port_type: "latency".into(),
                }],
                uri: Some(String::from(SRC_URI)),
                configuration: config.clone(),
                tags: vec![],
            };
            let snk = SinkDescriptor {
                id: "sink".into(),
                inputs: vec![PortDescriptor {
                    port_id: PORT.into(),
                    port_type: "latency".into(),
                }],
                uri: Some(String::from(SNK_URI)),
                configuration: config,
                tags: vec![],
            };
            (src, snk)
        }
    };

    // Adding source and sinks to descriptor
    dfd.sources.push(source_descriptor);
    dfd.sinks.push(sink_descriptor);

    // Creating and adding operators to descriptors

    for i in 0..args.pipeline {
        let op_descriptor = SimpleOperatorDescriptor {
            id: format!("op-{i}").into(),
            inputs: vec![PortDescriptor {
                port_id: PORT.into(),
                port_type: "latency".into(),
            }],
            outputs: vec![PortDescriptor {
                port_id: PORT.into(),
                port_type: "latency".into(),
            }],
            uri: Some(String::from(NOOP_URI)),
            configuration: None,
            tags: vec![],
        };
        dfd.operators.push(op_descriptor);
    }

    // Creating and adding source to first operator link

    let source_op0_link = LinkDescriptor {
        from: OutputDescriptor {
            node: "source".into(),
            output: PORT.into(),
        },
        to: InputDescriptor {
            node: "op-0".into(),
            input: PORT.into(),
        },
        size: None,
        queueing_policy: None,
        priority: None,
    };

    dfd.links.push(source_op0_link);

    // Creating and adding pipeline links

    for i in 1..args.pipeline {
        let j = i - 1;
        let opi_opj_link = LinkDescriptor {
            from: OutputDescriptor {
                node: format!("op-{j}").into(),
                output: PORT.into(),
            },
            to: InputDescriptor {
                node: format!("op-{i}").into(),
                input: PORT.into(),
            },
            size: None,
            queueing_policy: None,
            priority: None,
        };
        dfd.links.push(opi_opj_link);
    }

    // Creating and adding source to last operator link

    let last = args.pipeline - 1;

    let opn_sink_link = LinkDescriptor {
        from: OutputDescriptor {
            node: format!("op-{last}").into(),
            output: PORT.into(),
        },
        to: InputDescriptor {
            node: "sink".into(),
            input: PORT.into(),
        },
        size: None,
        queueing_policy: None,
        priority: None,
    };

    dfd.links.push(opn_sink_link);

    // Creating static mapping
    mapping.insert("source".into(), "src".into());
    mapping.insert("sink".into(), "snk".into());
    for i in 0..args.pipeline {
        mapping.insert(format!("op-{i}").into(), format!("comp{i}").into());
    }

    dfd.mapping = Some(mapping);

    let yaml_descriptor = dfd.to_yaml().unwrap();
    println!("Descriptor:\n{yaml_descriptor}");

    zenoh_flow_perf::write_string_to_file(yaml_descriptor, &args.descriptor_file);
}
