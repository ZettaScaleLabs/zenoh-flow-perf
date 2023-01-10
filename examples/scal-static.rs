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
use std::sync::Arc;
use zenoh::prelude::r#async::*;
use zenoh_flow::model::descriptor::{InputDescriptor, OutputDescriptor};
use zenoh_flow::model::record::{OperatorRecord, PortRecord, SinkRecord, SourceRecord};
use zenoh_flow::runtime::dataflow::instance::DataFlowInstance;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow_perf::nodes::{
    NoOpFactory, ScalNoOpFactory, ScalPingSourceFactory, ScalPongSinkFactory, LAT_PORT,
};

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
    factor: u32,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
}

// Run dataflow in single runtime
#[async_std::main]
async fn main() {
    let args = CallArgs::parse();

    let interval = 1.0 / (args.msgs as f64);

    let total_nodes: u32 = 1 << args.factor;

    let config = match args.kind {
        FanKind::In => Some(
            serde_json::json!({"interval" : interval, "nodes": total_nodes, "inputs": total_nodes, "msgs": args.msgs, "mode": 1, "multi":false}),
        ),
        FanKind::Out | FanKind::OutIn => Some(
            serde_json::json!({"interval" : interval, "nodes": total_nodes, "inputs": total_nodes, "msgs": args.msgs, "mode": 2, "multi":false}),
        ),
    };

    let session = Arc::new(
        zenoh::open(zenoh::config::Config::default())
            .res()
            .await
            .unwrap(),
    );
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());

    let rt_uuid = uuid::Uuid::new_v4();
    let runtime_name: Arc<str> = format!("scal-static-runtime-{}", rt_uuid).into();
    let ctx = RuntimeContext {
        session,
        hlc,
        loader: Arc::new(Loader::new(LoaderConfig::new())),
        runtime_name: runtime_name.clone(),
        runtime_uuid: rt_uuid,
    };

    let mut zf_graph = zenoh_flow::runtime::dataflow::DataFlow::new("scal-static", ctx.clone());

    /*
     * Source
     */
    let source_record = SourceRecord {
        id: "source".into(),
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

    zf_graph.add_source_factory(source_record, Arc::new(ScalPingSourceFactory));

    /*
     * Operators
     */
    for i in 0..total_nodes {
        let uid = 10 * i + 20u32;
        let operator_record = OperatorRecord {
            id: format!("op-{i}").into(),
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

    match args.kind {
        FanKind::In => {
            /*
             * Sink
             */
            let mut id_hm: HashMap<String, Value> = HashMap::new();
            id_hm.insert("id".to_string(), Value::Number(0.into()));
            let sink_config = Some(zenoh_flow_perf::nodes::dict_merge(
                &config.clone().unwrap(),
                &id_hm,
            ));

            let sink_record = SinkRecord {
                id: "sink".into(),
                uid: 10u32,
                inputs: vec![PortRecord {
                    port_id: LAT_PORT.into(),
                    port_type: "lat".into(),
                    uid: 11u32,
                }],
                uri: None,
                configuration: sink_config,
                runtime: runtime_name.clone(),
            };

            zf_graph.add_sink_factory(
                sink_record,
                Arc::new(ScalPongSinkFactory {
                    locator_tcp_port: 75389,
                }),
            );

            /*
             * Last special operator
             */
            let uid = 20 + total_nodes * 10;

            let mut inputs = vec![];
            let mut port_ids = vec![];
            for i in 0..total_nodes {
                let port_id = format!("{}{}", LAT_PORT, i);

                inputs.push(PortRecord {
                    port_id: port_id.clone().into(),
                    port_type: "lat".into(),
                    uid: uid + i,
                });

                port_ids.push(Value::String(port_id));
            }

            let port_ids_hm = HashMap::from([("port_ids".to_owned(), Value::Array(port_ids))]);
            let last_op_config = Some(zenoh_flow_perf::nodes::dict_merge(
                &config.clone().unwrap(),
                &port_ids_hm,
            ));

            let operator_record = OperatorRecord {
                id: "op-last".into(),
                uid,
                inputs,
                outputs: vec![PortRecord {
                    port_id: LAT_PORT.into(),
                    port_type: "lat".into(),
                    uid: uid + total_nodes,
                }],
                uri: None,
                configuration: last_op_config,
                runtime: runtime_name.clone(),
            };

            zf_graph.add_operator_factory(operator_record, Arc::new(ScalNoOpFactory));

            /*
             * Links
             */
            zf_graph.add_link(
                OutputDescriptor {
                    node: "op-last".into(),
                    output: LAT_PORT.into(),
                },
                InputDescriptor {
                    node: "sink".into(),
                    input: LAT_PORT.into(),
                },
            );

            for i in 0..total_nodes {
                zf_graph.add_link(
                    OutputDescriptor {
                        node: "source".into(),
                        output: LAT_PORT.into(),
                    },
                    InputDescriptor {
                        node: format!("op-{i}").into(),
                        input: LAT_PORT.into(),
                    },
                );

                zf_graph.add_link(
                    OutputDescriptor {
                        node: format!("op-{i}").into(),
                        output: LAT_PORT.into(),
                    },
                    InputDescriptor {
                        node: "op-last".into(),
                        input: format!("{}{}", LAT_PORT, i).into(),
                    },
                );
            }
        }
        FanKind::Out => {
            /*
             * Sinks
             */
            for i in 0..total_nodes {
                let uid = 10 + 2 * i;
                let mut id_hm: HashMap<String, Value> = HashMap::new();
                // CAVEAT: This `id` is used by the Sink to declare its publisher. Its value should
                // be equal to `i`.
                id_hm.insert("id".to_string(), i.into());
                let sink_config = Some(zenoh_flow_perf::nodes::dict_merge(
                    &config.clone().unwrap(),
                    &id_hm,
                ));

                let sink_record = SinkRecord {
                    id: format!("sink-{i}").into(),
                    uid,
                    inputs: vec![PortRecord {
                        port_id: LAT_PORT.into(),
                        port_type: "lat".into(),
                        uid: uid + 1,
                    }],
                    uri: None,
                    configuration: sink_config,
                    runtime: runtime_name.clone(),
                };

                zf_graph.add_sink_factory(
                    sink_record,
                    Arc::new(ScalPongSinkFactory {
                        locator_tcp_port: 32894 + i as usize,
                    }),
                );
            }

            /*
             * Links
             */
            for i in 0..total_nodes {
                // link src to op
                zf_graph.add_link(
                    OutputDescriptor {
                        node: "source".into(),
                        output: LAT_PORT.into(),
                    },
                    InputDescriptor {
                        node: format!("op-{i}").into(),
                        input: LAT_PORT.into(),
                    },
                );

                // link op to sink
                zf_graph.add_link(
                    OutputDescriptor {
                        node: format!("op-{i}").into(),
                        output: LAT_PORT.into(),
                    },
                    InputDescriptor {
                        node: format!("sink-{i}").into(),
                        input: LAT_PORT.into(),
                    },
                );
            }
        }
        _ => panic!("Not yet implemented..."),
    }

    // run the dataflow graph

    let mut instance = DataFlowInstance::try_instantiate(zf_graph, ctx.hlc.clone())
        .await
        .unwrap();

    for id in instance.get_sinks() {
        instance.start_node(&id).unwrap()
    }

    for id in instance.get_operators() {
        instance.start_node(&id).unwrap()
    }

    for id in instance.get_connectors() {
        instance.start_node(&id).unwrap()
    }

    for id in instance.get_sources() {
        instance.start_node(&id).unwrap()
    }

    std::future::pending::<()>().await;
}
