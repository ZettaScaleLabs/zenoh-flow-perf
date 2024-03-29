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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zenoh::prelude::r#async::*;
use zenoh::publication::CongestionControl;
use zenoh_flow::prelude::*;

use super::{LAT_PORT, THR_PORT};

/*
 ***************************************************************************************************
 *
 * LATENCY SINK
 *
 ***************************************************************************************************
 */

pub struct LatSink {
    input: Input<Latency>,
    pipeline: u64,
    messages: u64,
}

#[async_trait::async_trait]
impl Node for LatSink {
    async fn iteration(&self) -> Result<()> {
        if let Ok((Message::Data(data), _)) = self.input.recv().await {
            let now = get_epoch_us();
            let elapsed = now - (*data).ts;
            println!(
                "zenoh-flow,single,latency,{},8,{},{elapsed},us",
                self.pipeline, self.messages
            );
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Sink for LatSink {
    async fn new(
        _context: Context,
        configuration: Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Self> {
        let pipeline = match &configuration {
            Some(conf) => conf["pipeline"].as_u64().unwrap(),
            None => 1,
        };

        let messages = match &configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1,
        };

        let input = inputs.take(LAT_PORT).unwrap();

        Ok(LatSink {
            input,
            pipeline,
            messages,
        })
    }
}

/*
 ***************************************************************************************************
 *
 * PONG SINK
 *
 ***************************************************************************************************
 */

#[derive(Debug, Clone)]
struct PongSinkState {
    pipeline: u64,
    _interval: f64,
    msgs: u64,
    session: Arc<zenoh::Session>,
    expr: KeyExpr<'static>,
    data: Vec<u8>,
    layer: String,
}

pub struct PongSink {
    input: Input<Latency>,
    state: PongSinkState,
}

#[async_trait]
impl Node for PongSink {
    async fn iteration(&self) -> Result<()> {
        if let Ok((Message::Data(data), _)) = self.input.recv().await {
            let now = get_epoch_us();

            let elapsed = now - (*data).ts;
            let msgs = &self.state.msgs;
            let pipeline = &self.state.pipeline;
            let layer = &self.state.layer;
            println!("zenoh-flow,{layer},latency,{pipeline},8,{msgs},{elapsed},us");

            self.state
                .session
                .put(&self.state.expr, self.state.data.clone())
                .congestion_control(CongestionControl::Block)
                .res()
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Sink for PongSink {
    async fn new(
        _ctx: Context,
        configuration: Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Self> {
        let interval = match &configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let pipeline = match &configuration {
            Some(conf) => conf["pipeline"].as_u64().unwrap(),
            None => 1u64,
        };

        let msgs = match &configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1u64,
        };

        let multi = match &configuration {
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
        let session = zenoh::open(config).res().await.unwrap().into_arc();

        let expr = session
            .declare_keyexpr("test/latency/zf/pong")
            .res()
            .await
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

        let input = inputs.take(LAT_PORT).unwrap();
        Ok(PongSink { input, state })
    }
}

/*
 ***************************************************************************************************
 *
 * THROUGHPUT SINK
 *
 ***************************************************************************************************
 */

#[derive(Debug, Clone)]
struct ThrSinkState {
    pub _payload_size: usize,
    pub accumulator: Arc<AtomicUsize>,
    pub _abort_handle: AbortHandle,
}

pub struct ThrSink {
    input: InputRaw,
    state: Arc<ThrSinkState>,
}

#[async_trait]
impl Node for ThrSink {
    async fn iteration(&self) -> Result<()> {
        if let Ok(_link_message) = self.input.recv().await {
            self.state.accumulator.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[async_trait]
impl Sink for ThrSink {
    async fn new(
        _ctx: Context,
        configuration: Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Self> {
        let payload_size = match &configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let multi = match &configuration {
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

        let state = Arc::new(ThrSinkState {
            _payload_size: payload_size,
            accumulator,
            _abort_handle: abort_handle,
        });

        let input = inputs.take_raw(THR_PORT).unwrap();

        Ok(ThrSink { input, state })
    }
}

/*
 ***************************************************************************************************
 *
 * PONG SINK SCALABILITY
 *
 ***************************************************************************************************
 */

#[derive(Debug, Clone)]
struct ScalPongSinkState {
    nodes: u64,
    _interval: f64,
    msgs: u64,
    session: Arc<zenoh::Session>,
    expr: KeyExpr<'static>,
    data: Vec<u8>,
    diff: bool,
    layer: String,
}

pub struct ScalPongSink {
    input: Input<Latency>,
    state: Arc<ScalPongSinkState>,
}

#[async_trait]
impl Node for ScalPongSink {
    async fn iteration(&self) -> Result<()> {
        if let Ok((Message::Data(data), _)) = self.input.recv().await {
            let now = get_epoch_us();

            let elapsed = match self.state.diff {
                true => now - (*data).ts,
                false => (*data).ts,
            };
            let msgs = self.state.msgs;
            let pipeline = self.state.nodes;
            let layer = &self.state.layer;
            println!("zenoh-flow,{layer},latency,{pipeline},8,{msgs},{elapsed},us");

            self.state
                .session
                .put(&self.state.expr, self.state.data.clone())
                .congestion_control(CongestionControl::Block)
                .res()
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Sink for ScalPongSink {
    async fn new(
        _ctx: Context,
        configuration: Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Self> {
        let interval = match &configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let nodes = match &configuration {
            Some(conf) => conf["nodes"].as_u64().unwrap(),
            None => 1u64,
        };

        let msgs = match &configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1u64,
        };

        let node_id = match &configuration {
            Some(conf) => conf["id"].as_u64().unwrap(),
            None => 0u64,
        };

        let mode = match &configuration {
            Some(conf) => conf["mode"].as_u64().unwrap(),
            None => 1u64,
        };

        let multi = match &configuration {
            Some(conf) => conf["multi"].as_bool().unwrap(),
            None => false,
        };

        let locator_tcp_port = match &configuration {
            Some(conf) => conf["locator_tcp_port"].as_u64().unwrap(),
            None => 8448,
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

        let locator = format!("tcp/127.0.0.1:{}", locator_tcp_port);
        config
            .listen
            .set_endpoints(vec![locator.parse().unwrap()])
            .unwrap();
        let session = zenoh::open(config).res().await.unwrap().into_arc();

        let expr = session
            .declare_keyexpr(format!("test/latency/zf/pong/{node_id}"))
            .res()
            .await
            .unwrap();

        let state = Arc::new(ScalPongSinkState {
            _interval: interval,
            nodes,
            msgs,
            session,
            expr,
            data: vec![],
            diff,
            layer,
        });

        let input = inputs.take(LAT_PORT).unwrap();

        Ok(ScalPongSink { input, state })
    }
}
