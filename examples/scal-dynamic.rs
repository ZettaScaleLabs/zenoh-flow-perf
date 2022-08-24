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
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use zenoh_flow::model::dataflow::descriptor::FlattenDataFlowDescriptor;
use zenoh_flow::model::link::{LinkDescriptor, PortDescriptor};
use zenoh_flow::model::node::{SimpleOperatorDescriptor, SinkDescriptor, SourceDescriptor};
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::types::{NodeId, RuntimeId};
use zenoh_flow_perf::nodes::LAT_PORT;
use zenoh_flow_perf::runtime::Descriptor;
static DEFAULT_FACTOR: &str = "0";
static DEFAULT_FANKIND: &str = "in";
static DEFAULT_MSGS: &str = "1";
static DEFAULT_RT_NAME: &str = "nothing";
static DEFAULT_RT_DESCRIPTOR: &str = "./descriptor.yaml";

static NOOP_URI: &str = "file://./target/release/examples/libdyn_noop.so";
static LASTOP_URI: &str = "file://./target/release/examples/libdyn_op_last.so";

static PING_SRC_URI: &str = "file://./target/release/examples/libdyn_ping.so";
static PONG_SNK_URI: &str = "file://./target/release/examples/libdyn_pong_scal.so";

type ParseError = &'static str;

#[derive(Parser, Debug)]
enum FanKind {
    In,
    Out,
    OutIn,
}

impl FromStr for FanKind {
    type Err = ParseError;
    fn from_str(fan: &str) -> Result<Self, Self::Err> {
        match fan {
            "in" => Ok(FanKind::In),
            "out" => Ok(FanKind::Out),
            "outin" => Ok(FanKind::OutIn),
            _ => Err("Could not parse kind"),
        }
    }
}

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_FANKIND)]
    kind: FanKind,
    #[clap(short, long, default_value = DEFAULT_FACTOR, help = "Scaling factor N, nodes will be 2^N")]
    factor: u64,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
    #[clap(short, long)]
    runtime: bool,
    #[clap(short, long, default_value = DEFAULT_RT_NAME)]
    name: String,
    #[clap(short, long, default_value = DEFAULT_RT_DESCRIPTOR)]
    descriptor_file: String,
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

    let total_nodes: u64 = 1 << args.factor;

    let mode: usize = match args.kind {
        FanKind::In => 1,
        FanKind::Out | FanKind::OutIn => 2,
    };

    let config = Some(serde_json::json!({
        "interval": interval,
        "nodes": total_nodes,
        "inputs": total_nodes,
        "msgs": args.msgs,
        "mode": mode,
        "multi": true
    }));

    let mut mapping: HashMap<NodeId, RuntimeId> = HashMap::new();
    let mut dfd = FlattenDataFlowDescriptor {
        flow: format!("scaling{}", total_nodes),
        operators: vec![],
        sources: vec![],
        sinks: vec![],
        links: vec![],
        mapping: None,
        global_configuration: None,
        flags: None,
    };

    // Source and Sink

    let source_descriptor = SourceDescriptor {
        id: "source".into(),
        outputs: vec![PortDescriptor {
            port_id: LAT_PORT.into(),
            port_type: "latency".into(),
        }],
        uri: Some(String::from(PING_SRC_URI)),
        configuration: config.clone(),
        tags: vec![],
        flags: None
    };

    // Adding source and sinks to descriptor
    dfd.sources.push(source_descriptor);

    match args.kind {
        FanKind::In => {
            let mut id_hm: HashMap<String, Value> = HashMap::new();
            id_hm.insert("id".to_string(), serde_json::to_value::<usize>(0).unwrap());
            let sink_config = Some(zenoh_flow_perf::nodes::dict_merge(&config.unwrap(), &id_hm));

            // creating sink
            let sink_descriptor = SinkDescriptor {
                id: "sink".into(),
                inputs: vec![PortDescriptor {
                    port_id: LAT_PORT.into(),
                    port_type: "latency".into(),
                }],
                uri: Some(String::from(PONG_SNK_URI)),
                configuration: sink_config,
                tags: vec![],
                flags: None
            };
            dfd.sinks.push(sink_descriptor);
            mapping.insert("sink".into(), "snk".into());

            // creating nodes
            for i in 0..total_nodes {
                let op_descriptor = SimpleOperatorDescriptor {
                    id: format!("op-{i}").into(),
                    inputs: vec![PortDescriptor {
                        port_id: LAT_PORT.into(),
                        port_type: "latency".into(),
                    }],
                    outputs: vec![PortDescriptor {
                        port_id: LAT_PORT.into(),
                        port_type: "latency".into(),
                    }],
                    uri: Some(String::from(NOOP_URI)),
                    configuration: None,
                    tags: vec![],
                    flags: None
                };
                dfd.operators.push(op_descriptor);
            }

            // creating ports
            let mut inputs = vec![];
            for i in 0..total_nodes {
                inputs.push(PortDescriptor {
                    port_id: format!("{}{}", LAT_PORT, i).into(),
                    port_type: "latency".into(),
                });
            }

            // creating last operator (fan-in)
            let op_descriptor = SimpleOperatorDescriptor {
                id: "op-last".into(),
                inputs,
                outputs: vec![PortDescriptor {
                    port_id: LAT_PORT.into(),
                    port_type: "latency".into(),
                }],
                uri: Some(String::from(LASTOP_URI)),
                configuration: None,
                tags: vec![],
                flags: None
            };

            dfd.operators.push(op_descriptor);
            mapping.insert("op-last".into(), "complast".into());

            let op_last_snk_link = LinkDescriptor {
                from: OutputDescriptor {
                    node: "op-last".into(),
                    output: LAT_PORT.into(),
                },
                to: InputDescriptor {
                    node: "sink".into(),
                    input: LAT_PORT.into(),
                },
                size: None,
                queueing_policy: None,
                priority: None,
            };
            dfd.links.push(op_last_snk_link);

            // operators
            for i in 0..total_nodes {
                let src_op_link = LinkDescriptor {
                    from: OutputDescriptor {
                        node: "source".into(),
                        output: LAT_PORT.into(),
                    },
                    to: InputDescriptor {
                        node: format!("op-{i}").into(),
                        input: LAT_PORT.into(),
                    },
                    size: None,
                    queueing_policy: None,
                    priority: None,
                };

                let op_op_link = LinkDescriptor {
                    from: OutputDescriptor {
                        node: format!("op-{i}").into(),
                        output: LAT_PORT.into(),
                    },
                    to: InputDescriptor {
                        node: "op-last".into(),
                        input: format!("{}{}", LAT_PORT, i).into(),
                    },
                    size: None,
                    queueing_policy: None,
                    priority: None,
                };

                dfd.links.push(src_op_link);
                dfd.links.push(op_op_link);
            }
        }
        FanKind::Out => {
            // Creating operators
            for i in 0..total_nodes {
                let op_descriptor = SimpleOperatorDescriptor {
                    id: format!("op-{i}").into(),
                    inputs: vec![PortDescriptor {
                        port_id: LAT_PORT.into(),
                        port_type: "latency".into(),
                    }],
                    outputs: vec![PortDescriptor {
                        port_id: LAT_PORT.into(),
                        port_type: "latency".into(),
                    }],
                    uri: Some(String::from(NOOP_URI)),
                    configuration: None,
                    tags: vec![],
                    flags: None
                };
                dfd.operators.push(op_descriptor);
            }

            // creating sinks
            for i in 0..total_nodes {
                let mut id_hm: HashMap<String, Value> = HashMap::new();
                id_hm.insert("id".to_string(), i.into());
                let sink_config =
                    zenoh_flow_perf::nodes::dict_merge(&config.clone().unwrap(), &id_hm);

                let sink_descriptor = SinkDescriptor {
                    id: format!("sink-{i}").into(),
                    inputs: vec![PortDescriptor {
                        port_id: LAT_PORT.into(),
                        port_type: "latency".into(),
                    }],
                    uri: Some(String::from(PONG_SNK_URI)),
                    configuration: Some(sink_config),
                    tags: vec![],
                    flags: None
                };

                dfd.sinks.push(sink_descriptor);
            }

            // sink mapping
            for i in 0..total_nodes {
                mapping.insert(format!("sink-{i}").into(), format!("snk{i}").into());
            }

            // creating links
            for i in 0..total_nodes {
                let src_op_link = LinkDescriptor {
                    from: OutputDescriptor {
                        node: "source".into(),
                        output: LAT_PORT.into(),
                    },
                    to: InputDescriptor {
                        node: format!("op-{i}").into(),
                        input: LAT_PORT.into(),
                    },
                    size: None,
                    queueing_policy: None,
                    priority: None,
                };

                let op_snk_link = LinkDescriptor {
                    from: OutputDescriptor {
                        node: format!("op-{i}").into(),
                        output: LAT_PORT.into(),
                    },
                    to: InputDescriptor {
                        node: format!("sink-{i}").into(),
                        input: LAT_PORT.into(),
                    },
                    size: None,
                    queueing_policy: None,
                    priority: None,
                };

                dfd.links.push(src_op_link);
                dfd.links.push(op_snk_link);
            }
        }
        _ => panic!("Not yet implemented..."),
    }

    // Creating static mapping
    mapping.insert("source".into(), "src".into());
    for i in 0..total_nodes {
        mapping.insert(format!("op-{i}").into(), format!("comp{i}").into());
    }

    dfd.mapping = Some(mapping);
    let yaml_descriptor = dfd.to_yaml().unwrap();
    // println!("Descriptor:\n{yaml_descriptor}");

    zenoh_flow_perf::write_string_to_file(yaml_descriptor, &args.descriptor_file);
}
