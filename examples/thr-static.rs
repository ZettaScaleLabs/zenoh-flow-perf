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
use futures::stream::{AbortHandle, Abortable};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zenoh::prelude::r#async::*;
use zenoh_flow::model::descriptor::{InputDescriptor, OutputDescriptor};
use zenoh_flow::model::record::{OperatorRecord, PortRecord, SinkRecord, SourceRecord};
use zenoh_flow::prelude::*;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;
use zenoh_flow::traits::{Node, OperatorFactory, SinkFactory, SourceFactory};
use zenoh_flow_perf::ThrData;

static DEFAULT_SIZE: &str = "8";
static DEFAULT_DURATION: &str = "60";

static OUTPUT: &str = "out";
static INPUT: &str = "in";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_SIZE)]
    size: u64,
    #[clap(short, long, default_value = DEFAULT_DURATION)]
    duration: u64,
}

/***************************************************************************************************
 *
 * SOURCE
 *
 **************************************************************************************************/
struct ThrSource {
    output: Output,
    data: Arc<ThrData>,
}

#[async_trait::async_trait]
impl Node for ThrSource {
    async fn iteration(&self) -> Result<()> {
        self.output
            .send_async(Data::from(Arc::clone(&self.data)), None)
            .await
    }
}

struct ThrSourceFactory;

#[async_trait::async_trait]
impl SourceFactory for ThrSourceFactory {
    async fn new_source(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Arc<dyn Node>>> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let data = Arc::new(ThrData {
            data: (0usize..payload_size)
                .map(|i| (i % 10) as u8)
                .collect::<Vec<u8>>(),
        });
        let output = outputs.take(OUTPUT).expect("No output `out`");

        Ok(Some(Arc::new(ThrSource { output, data })))
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
struct ThrSink {
    input: Input,
    accumulator: Arc<AtomicUsize>,
    _abort_handle: AbortHandle,
}

#[async_trait::async_trait]
impl Node for ThrSink {
    async fn iteration(&self) -> Result<()> {
        if let Ok(Message::Data(_)) = self.input.recv_async().await {
            self.accumulator.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }
}

struct ThrSinkFactory;

#[async_trait::async_trait]
impl SinkFactory for ThrSinkFactory {
    async fn new_sink(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Option<Arc<dyn Node>>> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let multi = match configuration {
            Some(conf) => conf["multi"].as_bool().unwrap(),
            None => false,
        };

        let scenario = match multi {
            true => "multi".to_string(),
            false => "single".to_string(),
        };

        let accumulator = Arc::new(AtomicUsize::new(0usize));
        let loop_accumulator = Arc::clone(&accumulator);
        let loop_payload_size = payload_size;

        let print_loop = async move {
            loop {
                let now = Instant::now();
                async_std::task::sleep(Duration::from_secs(1)).await;
                let elapsed = now.elapsed().as_micros() as f64;

                let c = loop_accumulator.swap(0, Ordering::Relaxed);
                if c > 0 {
                    let interval = 1_000_000.0 / elapsed;
                    let msgs = (c as f64 / interval).floor() as usize;
                    // framework, scenario, test, pipeline, payload, rate, value, unit
                    println!(
                        "zenoh-flow,{scenario},throughput,1,{loop_payload_size},{msgs},{msgs},msgs"
                    );

                    // println!(
                    //     // layer,scenario,test,name,messages,pipeline,latency,payload,unit
                    //     "zenoh-flow,{scenario},throughput,{test_name},{msgs},1,0,{loop_payload_size},msgs"
                    // );
                }
            }
        };

        let (_abort_handle, abort_registration) = AbortHandle::new_pair();
        let _print_task = async_std::task::spawn(Abortable::new(print_loop, abort_registration));

        let input = inputs.take(INPUT).unwrap();

        Ok(Some(Arc::new(ThrSink {
            input,
            _abort_handle,
            accumulator,
        })))
    }
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
    let runtime_name = format!("thr-runtime-{}", rt_uuid).into();
    let ctx = RuntimeContext {
        session,
        hlc,
        loader: Arc::new(Loader::new(LoaderConfig::new())),
        runtime_name: Arc::clone(&runtime_name),
        runtime_uuid: rt_uuid,
    };

    let mut zf_graph = zenoh_flow::runtime::dataflow::DataFlow2::new("thr-static", ctx.clone());

    let config = Some(serde_json::json!({"payload_size" : args.size, "multi": false}));

    let thr_source_record = SourceRecord {
        id: "thr-source".into(),
        uid: 0,
        outputs: vec![PortRecord {
            uid: 0,
            port_id: OUTPUT.into(),
            port_type: "thr".into(),
        }],
        uri: None,
        configuration: config.clone(),
        runtime: Arc::clone(&runtime_name),
    };

    zf_graph.add_source_factory("thr-source", thr_source_record, Arc::new(ThrSourceFactory));

    let thr_sink_record = SinkRecord {
        id: "thr-sink".into(),
        uid: 1,
        inputs: vec![PortRecord {
            uid: 1,
            port_id: INPUT.into(),
            port_type: "thr".into(),
        }],
        uri: None,
        configuration: config.clone(),
        runtime: Arc::clone(&runtime_name),
    };

    zf_graph.add_sink_factory("thr-sink", thr_sink_record, Arc::new(ThrSinkFactory));

    let no_op_record = OperatorRecord {
        id: "noop".into(),
        uid: 2,
        inputs: vec![PortRecord {
            uid: 2,
            port_id: INPUT.into(),
            port_type: "thr".into(),
        }],
        outputs: vec![PortRecord {
            uid: 2,
            port_id: OUTPUT.into(),
            port_type: "thr".into(),
        }],
        uri: None,
        configuration: config.clone(),
        runtime: Arc::clone(&runtime_name),
    };

    zf_graph.add_operator_factory("noop", no_op_record, Arc::new(NoOpFactory));

    zf_graph.add_link(
        OutputDescriptor {
            node: "thr-source".into(),
            output: OUTPUT.into(),
        },
        InputDescriptor {
            node: "noop".into(),
            input: INPUT.into(),
        },
    );

    zf_graph.add_link(
        OutputDescriptor {
            node: "noop".into(),
            output: OUTPUT.into(),
        },
        InputDescriptor {
            node: "thr-sink".into(),
            input: INPUT.into(),
        },
    );

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

    async_std::task::sleep(std::time::Duration::from_secs(args.duration)).await;
}
