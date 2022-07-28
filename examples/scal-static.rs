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

use async_std::sync::Arc;
use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use zenoh_flow::model::link::PortDescriptor;
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::runtime::dataflow::instance::DataflowInstance;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow_perf::nodes::{IRNoOp, NoOp, ScalPingSource, ScalPongSink, LAT_PORT};

static DEFAULT_FACTOR: &str = "0";
static DEFAULT_FANKIND: &str = "in";
static DEFAULT_MSGS: &str = "1";

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
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    let args = CallArgs::parse();

    let interval = 1.0 / (args.msgs as f64);

    let total_nodes: u64 = 1 << args.factor;

    let config = match args.kind {
        FanKind::In => Some(
            serde_json::json!({"interval" : interval, "nodes": total_nodes, "inputs": total_nodes, "msgs": args.msgs, "mode": 1, "multi":false}),
        ),
        FanKind::Out | FanKind::OutIn => Some(
            serde_json::json!({"interval" : interval, "nodes": total_nodes, "inputs": total_nodes, "msgs": args.msgs, "mode": 2, "multi":false}),
        ),
    };

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
        zenoh_flow::runtime::dataflow::Dataflow::new(ctx.clone(), "scal-static".into(), None);

    // Source
    let source = Arc::new(ScalPingSource {});
    zf_graph
        .try_add_static_source(
            "source".into(),
            config.clone(),
            PortDescriptor {
                port_id: String::from(LAT_PORT).into(),
                port_type: String::from("lat").into(),
            },
            source,
        )
        .unwrap();

    match args.kind {
        FanKind::In => {
            let sink = Arc::new(ScalPongSink {});
            let mut id_hm: HashMap<String, Value> = HashMap::new();
            id_hm.insert("id".to_string(), Value::Number(0.into()));
            let sink_config = Some(zenoh_flow_perf::nodes::dict_merge(
                &config.clone().unwrap(),
                &id_hm,
            ));

            zf_graph
                .try_add_static_sink(
                    "sink".into(),
                    sink_config,
                    PortDescriptor {
                        port_id: String::from(LAT_PORT).into(),
                        port_type: String::from("lat").into(),
                    },
                    sink,
                )
                .unwrap();

            // creating operators
            for i in 0..total_nodes {
                let op = Arc::new(NoOp {});

                zf_graph
                    .try_add_static_operator(
                        format!("op-{i}").into(),
                        config.clone(),
                        vec![PortDescriptor {
                            port_id: String::from(LAT_PORT).into(),
                            port_type: String::from("lat").into(),
                        }],
                        vec![PortDescriptor {
                            port_id: String::from(LAT_PORT).into(),
                            port_type: String::from("lat").into(),
                        }],
                        op,
                    )
                    .unwrap();
            }

            // creating ports
            let mut inputs = vec![];
            for i in 0..total_nodes {
                inputs.push(PortDescriptor {
                    port_id: format!("{}{}", LAT_PORT, i).into(),
                    port_type: "lat".into(),
                });
            }

            // creating last operator (fan-in)

            let last_op = Arc::new(IRNoOp {});

            zf_graph
                .try_add_static_operator(
                    "op-last".into(),
                    config.clone(),
                    inputs,
                    vec![PortDescriptor {
                        port_id: String::from(LAT_PORT).into(),
                        port_type: String::from("lat").into(),
                    }],
                    last_op,
                )
                .unwrap();

            // last to sink link
            zf_graph
                .try_add_link(
                    OutputDescriptor {
                        node: "op-last".into(),
                        output: LAT_PORT.into(),
                    },
                    InputDescriptor {
                        node: "sink".into(),
                        input: LAT_PORT.into(),
                    },
                    None,
                    None,
                    None,
                )
                .unwrap();

            // operators
            for i in 0..total_nodes {
                // link src to op
                zf_graph
                    .try_add_link(
                        OutputDescriptor {
                            node: "source".into(),
                            output: LAT_PORT.into(),
                        },
                        InputDescriptor {
                            node: format!("op-{i}").into(),
                            input: LAT_PORT.into(),
                        },
                        None,
                        None,
                        None,
                    )
                    .unwrap();

                // link op to last-op

                zf_graph
                    .try_add_link(
                        OutputDescriptor {
                            node: format!("op-{i}").into(),
                            output: LAT_PORT.into(),
                        },
                        InputDescriptor {
                            node: "op-last".into(),
                            input: format!("{}{}", LAT_PORT, i).into(),
                        },
                        None,
                        None,
                        None,
                    )
                    .unwrap();
            }
        }
        FanKind::Out => {
            // Creating operators
            for i in 0..total_nodes {
                let op = Arc::new(NoOp {});

                zf_graph
                    .try_add_static_operator(
                        format!("op-{i}").into(),
                        config.clone(),
                        vec![PortDescriptor {
                            port_id: String::from(LAT_PORT).into(),
                            port_type: String::from("lat").into(),
                        }],
                        vec![PortDescriptor {
                            port_id: String::from(LAT_PORT).into(),
                            port_type: String::from("lat").into(),
                        }],
                        op,
                    )
                    .unwrap();
            }

            // creating sinks
            for i in 0..total_nodes {
                let mut id_hm: HashMap<String, Value> = HashMap::new();
                id_hm.insert("id".to_string(), i.into());
                let sink_config = Some(zenoh_flow_perf::nodes::dict_merge(
                    &config.clone().unwrap(),
                    &id_hm,
                ));

                let sink = Arc::new(ScalPongSink {});

                zf_graph
                    .try_add_static_sink(
                        format!("sink-{i}").into(),
                        sink_config.clone(),
                        PortDescriptor {
                            port_id: String::from(LAT_PORT).into(),
                            port_type: String::from("lat").into(),
                        },
                        sink,
                    )
                    .unwrap();
            }

            // creating links
            for i in 0..total_nodes {
                // link src to op
                zf_graph
                    .try_add_link(
                        OutputDescriptor {
                            node: "source".into(),
                            output: LAT_PORT.into(),
                        },
                        InputDescriptor {
                            node: format!("op-{i}").into(),
                            input: LAT_PORT.into(),
                        },
                        None,
                        None,
                        None,
                    )
                    .unwrap();

                // link op to sink
                zf_graph
                    .try_add_link(
                        OutputDescriptor {
                            node: format!("op-{i}").into(),
                            output: LAT_PORT.into(),
                        },
                        InputDescriptor {
                            node: format!("sink-{i}").into(),
                            input: LAT_PORT.into(),
                        },
                        None,
                        None,
                        None,
                    )
                    .unwrap();
            }
        }
        _ => panic!("Not yet implemented..."),
    }

    // run the dataflow graph

    let mut instance = DataflowInstance::try_instantiate(zf_graph, ctx.hlc.clone()).unwrap();

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

    std::future::pending::<()>().await;
}
