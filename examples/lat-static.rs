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
use std::{sync::Arc, time::Duration};
use zenoh::prelude::r#async::*;
use zenoh_flow::{
    model::descriptor::{InputDescriptor, OutputDescriptor},
    model::record::{OperatorRecord, PortRecord, SinkRecord, SourceRecord},
    prelude::*,
    runtime::{
        dataflow::loader::{Loader, LoaderConfig},
        RuntimeContext,
    },
    traits::{Node, OperatorFactory, SinkFactory, SourceFactory},
};
use zenoh_flow_perf::{get_epoch_us, Latency};

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";

static OUTPUT: &str = "out";
static INPUT: &str = "in";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u64,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
}

/***************************************************************************************************
 *
 * SOURCE
 *
 **************************************************************************************************/

struct LatSource {
    sleep_interval: Duration,
    output: Output,
}

#[async_trait::async_trait]
impl Node for LatSource {
    async fn iteration(&self) -> Result<()> {
        async_std::task::sleep(self.sleep_interval).await;
        let data = Data::from(Latency { ts: get_epoch_us() });
        self.output.send_async(data, None).await
    }
}

struct LatSourceFactory;

#[async_trait::async_trait]
impl SourceFactory for LatSourceFactory {
    async fn new_source(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Arc<dyn Node>>> {
        let interval = match configuration {
            Some(config) => config["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let sleep_interval = Duration::from_secs_f64(interval);
        let output = outputs.take(OUTPUT).expect("No output `out`");

        Ok(Some(Arc::new(LatSource {
            sleep_interval,
            output,
        })))
    }
}

/***************************************************************************************************
 *
 * OPERATOR
 *
 **************************************************************************************************/
struct NoOp {
    input: Input,
    output: Output,
}

#[async_trait::async_trait]
impl Node for NoOp {
    async fn iteration(&self) -> Result<()> {
        if let Ok(Message::Data(mut message)) = self.input.recv_async().await {
            self.output
                .send_async(message.get_inner_data().clone(), None)
                .await
                .unwrap();
        }

        Ok(())
    }
}

struct NoOpFactory;

#[async_trait::async_trait]
impl OperatorFactory for NoOpFactory {
    async fn new_operator(
        &self,
        _context: &mut Context,
        _configuration: &Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<Option<Arc<dyn Node>>> {
        Ok(Some(Arc::new(NoOp {
            input: inputs.take(INPUT).unwrap(),
            output: outputs.take(OUTPUT).unwrap(),
        })))
    }
}

/***************************************************************************************************
 *
 * SINK
 *
 **************************************************************************************************/

struct LatSink {
    input: Input,
    pipeline: u64,
    messages: u64,
}

#[async_trait::async_trait]
impl Node for LatSink {
    async fn iteration(&self) -> Result<()> {
        if let Ok(Message::Data(mut message)) = self.input.recv_async().await {
            let data = message.get_inner_data().try_get::<Latency>()?;
            let now = get_epoch_us();
            let elapsed = now - data.ts;
            println!(
                "zenoh-flow,single,latency,{},8,{},{elapsed},us",
                self.pipeline, self.messages
            );
        }

        Ok(())
    }
}

struct LatSinkFactory;

#[async_trait::async_trait]
impl SinkFactory for LatSinkFactory {
    async fn new_sink(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Option<Arc<dyn Node>>> {
        let pipeline = match configuration {
            Some(conf) => conf["pipeline"].as_u64().unwrap(),
            None => 1,
        };

        let messages = match configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1,
        };

        let input = inputs.take(INPUT).unwrap();

        Ok(Some(Arc::new(LatSink {
            input,
            pipeline,
            messages,
        })))
    }
}

/***************************************************************************************************
 *
 * STATIC DATA FLOW
 *
 **************************************************************************************************/

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
    let runtime_name = format!("thr-runtime-{}", rt_uuid).into();
    let ctx = RuntimeContext {
        session,
        hlc,
        loader: Arc::new(Loader::new(LoaderConfig::new())),
        runtime_name: Arc::clone(&runtime_name),
        runtime_uuid: rt_uuid,
    };

    let mut zf_graph = zenoh_flow::runtime::dataflow::DataFlow2::new("lat-static", ctx.clone());

    let config = Some(
        serde_json::json!({"interval" : interval, "pipeline":args.pipeline, "msgs": args.msgs, "multi":false}),
    );

    let lat_source_record = SourceRecord {
        id: "lat-source".into(),
        uid: 0,
        outputs: vec![PortRecord {
            uid: 0,
            port_id: OUTPUT.into(),
            port_type: "latency".into(),
        }],
        uri: None,
        configuration: config.clone(),
        runtime: Arc::clone(&runtime_name),
    };

    zf_graph.add_source_factory("lat-source", lat_source_record, Arc::new(LatSourceFactory));

    let lat_sink_record = SinkRecord {
        id: "lat-sink".into(),
        uid: 1,
        inputs: vec![PortRecord {
            uid: 1,
            port_id: INPUT.into(),
            port_type: "latency".into(),
        }],
        uri: None,
        configuration: config.clone(),
        runtime: Arc::clone(&runtime_name),
    };

    zf_graph.add_sink_factory("lat-sink", lat_sink_record, Arc::new(LatSinkFactory));

    for i in 0..args.pipeline {
        let uid = (i + 2) as usize;
        let no_op_record = OperatorRecord {
            id: format!("noop{i}").into(),
            uid,
            inputs: vec![PortRecord {
                uid,
                port_id: INPUT.into(),
                port_type: "latency".into(),
            }],
            outputs: vec![PortRecord {
                uid,
                port_id: OUTPUT.into(),
                port_type: "latency".into(),
            }],
            uri: None,
            configuration: config.clone(),
            runtime: Arc::clone(&runtime_name),
        };

        zf_graph.add_operator_factory(format!("noop{i}"), no_op_record, Arc::new(NoOpFactory));
    }

    let mut pipe = String::from("");
    zf_graph.add_link(
        OutputDescriptor {
            node: "lat-source".into(),
            output: OUTPUT.into(),
        },
        InputDescriptor {
            node: "noop0".into(),
            input: INPUT.into(),
        },
    );
    pipe.push_str("lat-source-->noop0-->");

    for i in 1..args.pipeline {
        // println!("Iteration {i}");

        let j = i - 1;
        zf_graph.add_link(
            OutputDescriptor {
                node: format!("noop{j}").into(),
                output: OUTPUT.into(),
            },
            InputDescriptor {
                node: format!("noop{i}").into(),
                input: INPUT.into(),
            },
        );
        pipe.push_str(format!("noop{j}-->noop{i}-->").as_str());
    }

    let len = args.pipeline - 1;
    zf_graph.add_link(
        OutputDescriptor {
            node: format!("noop{len}").into(),
            output: OUTPUT.into(),
        },
        InputDescriptor {
            node: "lat-sink".into(),
            input: INPUT.into(),
        },
    );
    pipe.push_str(format!("noop{len}-->lat-sink").as_str());

    // println!("Pipeline is: {pipe}");

    // let mut instance = DataflowInstance::try_instantiate(zf_graph, ctx.hlc.clone()).unwrap();
    let mut instance = zf_graph.try_instantiate(ctx.hlc.clone()).await.unwrap();

    let sinks: Vec<_> = instance.get_sinks().map(Arc::clone).collect();
    for sink_id in &sinks {
        instance.start_node(sink_id).await.unwrap();
    }

    let operators: Vec<_> = instance.get_operators().map(Arc::clone).collect();
    for operator_id in &operators {
        instance.start_node(operator_id).await.unwrap();
    }

    let sources: Vec<_> = instance.get_sources().map(Arc::clone).collect();
    for source_id in &sources {
        instance.start_node(source_id).await.unwrap();
    }

    // let mut connectors = instance.get_connectors();
    // for id in connectors.drain(..) {
    //     instance.start_node(&id).await.unwrap()
    // }

    std::future::pending::<()>().await;
}
