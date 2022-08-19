//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use crate::{get_epoch_us, Latency};
use async_trait::async_trait;
use futures::future::{AbortHandle, Abortable};
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zenoh::prelude::*;
use zenoh::publication::CongestionControl;
use zenoh_flow::prelude::*;

use super::{LAT_PORT, THR_PORT};

// Latency SINK
pub struct LatSink;

#[derive(Debug, Clone)]
struct LatSinkState {
    pipeline: u64,
    _interval: f64,
    msgs: u64,
}

#[async_trait]
impl Sink for LatSink {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Option<Arc<dyn AsyncIteration>>> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let pipeline = match configuration {
            Some(conf) => conf["pipeline"].as_u64().unwrap(),
            None => 1u64,
        };

        let msgs = match configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1u64,
        };

        let state = LatSinkState {
            _interval: interval,
            pipeline,
            msgs,
        };

        let input = inputs.remove(LAT_PORT).unwrap();

        Ok(Some(Arc::new(move || async move {
            if let Ok(Message::Data(mut msg)) = input.recv_async().await {
                let data = msg.get_inner_data().try_get::<Latency>()?;
                let now = get_epoch_us();

                let elapsed = now - data.ts;
                let msgs = state.msgs;
                let pipeline = state.pipeline;
                println!("zenoh-flow,single,latency,{pipeline},8,{msgs},{elapsed},us");
            }
            Ok(())
        })))
    }
}

// Pong SINK

pub struct PongSink;

#[derive(Debug, Clone)]
struct PongSinkState {
    pipeline: u64,
    _interval: f64,
    msgs: u64,
    session: Arc<zenoh::Session>,
    expr: ExprId,
    data: Vec<u8>,
    layer: String,
}

#[async_trait]
impl Sink for PongSink {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Option<Arc<dyn AsyncIteration>>> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let pipeline = match configuration {
            Some(conf) => conf["pipeline"].as_u64().unwrap(),
            None => 1u64,
        };

        let msgs = match configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1u64,
        };

        let multi = match configuration {
            Some(conf) => conf["multi"].as_bool().unwrap(),
            None => false,
        };

        let layer = match multi {
            true => "multi".to_string(),
            false => "single".to_string(),
        };

        let mut config = zenoh::config::Config::default();
        config
            .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
            .unwrap();
        let session = zenoh::open(config).wait().unwrap().into_arc();

        let expr = session
            .declare_expr("/test/latency/zf/pong")
            .wait()
            .unwrap();

        let state = PongSinkState {
            _interval: interval,
            pipeline,
            msgs,
            session,
            expr,
            data: vec![],
            layer,
        };

        let input = inputs.remove(LAT_PORT).unwrap();

        Ok(Some(Arc::new(move || async move {
            if let Ok(Message::Data(mut msg)) = input.recv_async().await {
                let data = msg.get_inner_data().try_get::<Latency>()?;
                let now = get_epoch_us();

                let elapsed = now - data.ts;
                let msgs = state.msgs;
                let pipeline = state.pipeline;
                let layer = state.layer;
                println!("zenoh-flow,{layer},latency,{pipeline},8,{msgs},{elapsed},us");

                state
                    .session
                    .put(&state.expr, state.data.clone())
                    .congestion_control(CongestionControl::Block)
                    .await?;
            }
            Ok(())
        })))
    }
}

// THR SINK

pub struct ThrSink;

#[derive(Debug, Clone)]
struct SinkState {
    pub _payload_size: usize,
    pub accumulator: Arc<AtomicUsize>,
    pub _abort_handle: AbortHandle,
}

#[async_trait]
impl Sink for ThrSink {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Option<Arc<dyn AsyncIteration>>> {
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

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let _print_task = async_std::task::spawn(Abortable::new(print_loop, abort_registration));

        let state = SinkState {
            _payload_size: payload_size,
            accumulator,
            _abort_handle: abort_handle,
        };

        let input = inputs.remove(THR_PORT).unwrap();

        Ok(Some(Arc::new(move || async move {
            if let Ok(Message::Data(_)) = input.recv_async().await {
                state.accumulator.fetch_add(1, Ordering::Relaxed);
            }
            Ok(())
        })))
    }
}

// PONG SINK SCALABILITY

pub struct ScalPongSink;

#[derive(Debug, Clone)]
struct ScalPongSinkState {
    nodes: u64,
    _interval: f64,
    msgs: u64,
    session: Arc<zenoh::Session>,
    expr: ExprId,
    data: Vec<u8>,
    diff: bool,
    layer: String,
}

#[async_trait]
impl Sink for ScalPongSink {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Option<Arc<dyn AsyncIteration>>> {
        let mut rng = rand::thread_rng();
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let nodes = match configuration {
            Some(conf) => conf["nodes"].as_u64().unwrap(),
            None => 1u64,
        };

        let msgs = match configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1u64,
        };

        let node_id = match configuration {
            Some(conf) => conf["id"].as_u64().unwrap(),
            None => 0u64,
        };

        let mode = match configuration {
            Some(conf) => conf["mode"].as_u64().unwrap(),
            None => 1u64,
        };

        let multi = match configuration {
            Some(conf) => conf["multi"].as_bool().unwrap(),
            None => false,
        };

        let layer = match multi {
            true => "multi".to_string(),
            false => "single".to_string(),
        };

        let diff = !matches!(mode, 1);

        let mut config = zenoh::config::Config::default();
        config
            .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
            .unwrap();

        let locator = format!("tcp/127.0.0.1:{}", rng.gen_range(8000..65000));
        config
            .listen
            .set_endpoints(vec![locator.parse().unwrap()])
            .unwrap();
        let session = zenoh::open(config).wait().unwrap().into_arc();

        let expr = session
            .declare_expr(format!("/test/latency/zf/pong/{node_id}"))
            .wait()
            .unwrap();

        let state = ScalPongSinkState {
            _interval: interval,
            nodes,
            msgs,
            session,
            expr,
            data: vec![],
            diff,
            layer,
        };

        let input = inputs.remove(LAT_PORT).unwrap();

        Ok(Some(Arc::new(move || async move {
            if let Ok(Message::Data(mut msg)) = input.recv_async().await {
                let data = msg.get_inner_data().try_get::<Latency>()?;
                let now = get_epoch_us();

                let elapsed = match state.diff {
                    true => now - data.ts,
                    false => data.ts,
                };
                let msgs = state.msgs;
                let pipeline = state.nodes;
                let layer = &state.layer;
                println!("zenoh-flow,{layer},latency,{pipeline},8,{msgs},{elapsed},us");

                state
                    .session
                    .put(&state.expr, state.data.clone())
                    .congestion_control(CongestionControl::Block)
                    .await?;
            }
            Ok(())
        })))
    }
}
