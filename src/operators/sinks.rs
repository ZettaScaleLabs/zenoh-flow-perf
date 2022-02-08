use crate::{get_epoch_us, Latency};

use async_trait::async_trait;
use futures::future::{AbortHandle, Abortable};
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use zenoh::prelude::*;
use zenoh::publication::CongestionControl;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{Configuration, Context, Node, Sink, State, ZFResult};

// Latency SINK
pub struct LatSink;

#[derive(ZFState, Debug, Clone)]
struct LatSinkState {
    pipeline: u64,
    interval: f64,
    msgs: u64,
}

#[async_trait]
impl Sink for LatSink {
    async fn run(
        &self,
        _context: &mut Context,
        state: &mut State,
        mut input: zenoh_flow::runtime::message::DataMessage,
    ) -> zenoh_flow::ZFResult<()> {
        let real_state = state.try_get::<LatSinkState>()?;
        let _ = real_state.interval;

        let data = input.get_inner_data().try_get::<Latency>()?;

        let now = get_epoch_us();

        let elapsed = now - data.ts;
        let msgs = real_state.msgs;
        let pipeline = real_state.pipeline;
        // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
        println!("zenoh-flow,scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");

        Ok(())
    }
}

impl Node for LatSink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
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

        Ok(State::from(LatSinkState {
            interval,
            pipeline,
            msgs,
        }))
    }

    fn finalize(&self, state: &mut State) -> ZFResult<()> {
        let _real_state = state.try_get::<LatSinkState>()?;

        Ok(())
    }
}

// Pong SINK

pub struct PongSink;

#[derive(ZFState, Debug, Clone)]
struct PongSinkState {
    pipeline: u64,
    interval: f64,
    msgs: u64,
    session: Arc<zenoh::Session>,
    expr: ExprId,
    data: Vec<u8>,
}

#[async_trait]
impl Sink for PongSink {
    async fn run(
        &self,
        _context: &mut Context,
        state: &mut State,
        mut input: zenoh_flow::runtime::message::DataMessage,
    ) -> zenoh_flow::ZFResult<()> {
        let real_state = state.try_get::<PongSinkState>()?;
        let _ = real_state.interval;

        let data = input.get_inner_data().try_get::<Latency>()?;

        let now = get_epoch_us();

        let elapsed = now - data.ts;
        let msgs = real_state.msgs;
        let pipeline = real_state.pipeline;
        // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
        println!("zenoh-flow,scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");

        // pong back
        real_state
            .session
            .put(&real_state.expr, real_state.data.clone())
            .congestion_control(CongestionControl::Block)
            .await?;

        Ok(())
    }
}

impl Node for PongSink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
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

        let mut config = zenoh::config::Config::default();
        config
            .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
            .unwrap();
        let session = zenoh::open(config).wait().unwrap().into_arc();

        let expr = session
            .declare_expr("/test/latency/zf/pong")
            .wait()
            .unwrap();

        Ok(State::from(PongSinkState {
            interval,
            pipeline,
            msgs,
            session,
            expr,
            data: vec![],
        }))
    }

    fn finalize(&self, state: &mut State) -> ZFResult<()> {
        let _real_state = state.try_get::<PongSinkState>()?;

        Ok(())
    }
}

// THR SINK

pub struct ThrSink;

#[derive(ZFState, Debug, Clone)]
struct SinkState {
    pub _payload_size: usize,
    pub accumulator: Arc<AtomicUsize>,
    pub abort_handle: AbortHandle,
}

#[async_trait]
impl Sink for ThrSink {
    async fn run(
        &self,
        _context: &mut Context,
        state: &mut State,
        _input: zenoh_flow::runtime::message::DataMessage,
    ) -> zenoh_flow::ZFResult<()> {
        let my_state = state.try_get::<SinkState>()?;
        my_state.accumulator.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl Node for ThrSink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let accumulator = Arc::new(AtomicUsize::new(0usize));

        let loop_accumulator = Arc::clone(&accumulator);
        let loop_payload_size = payload_size.clone();

        let print_loop = async move {
            // println!("layer,scenario,test,name,size,messages");
            loop {
                let now = Instant::now();
                async_std::task::sleep(Duration::from_secs(1)).await;
                let elapsed = now.elapsed().as_micros() as f64;

                let c = loop_accumulator.swap(0, Ordering::Relaxed);
                if c > 0 {
                    let interval = 1_000_000.0 / elapsed;
                    println!(
                        "zenoh-flow-static,same-runtime,throughput,{},{},{}",
                        "test-name",
                        loop_payload_size,
                        (c as f64 / interval).floor() as usize
                    );
                }
            }
        };

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let _print_task = async_std::task::spawn(Abortable::new(print_loop, abort_registration));

        Ok(State::from(SinkState {
            _payload_size: payload_size,
            accumulator,
            abort_handle,
        }))
    }

    fn finalize(&self, state: &mut State) -> ZFResult<()> {
        let real_state = state.try_get::<SinkState>()?;

        real_state.abort_handle.abort();
        Ok(())
    }
}

// PONG SINK SCALABILITY

pub struct ScalPongSink;

#[derive(ZFState, Debug, Clone)]
struct ScalPongSinkState {
    nodes: u64,
    interval: f64,
    msgs: u64,
    session: Arc<zenoh::Session>,
    expr: ExprId,
    data: Vec<u8>,
}

#[async_trait]
impl Sink for ScalPongSink {
    async fn run(
        &self,
        _context: &mut Context,
        state: &mut State,
        mut input: zenoh_flow::runtime::message::DataMessage,
    ) -> zenoh_flow::ZFResult<()> {
        let real_state = state.try_get::<ScalPongSinkState>()?;
        let _ = real_state.interval;

        let data = input.get_inner_data().try_get::<Latency>()?;

        let elapsed = data.ts;
        let msgs = real_state.msgs;
        let pipeline = real_state.nodes;
        // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
        println!("zenoh-flow-multi,scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");

        // pong back
        real_state
            .session
            .put(&real_state.expr, real_state.data.clone())
            .congestion_control(CongestionControl::Block)
            .await?;

        Ok(())
    }
}

impl Node for ScalPongSink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
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

        let mut config = zenoh::config::Config::default();
        config
            .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
            .unwrap();

        let locator = format!("tcp/127.0.0.1:{}", rng.gen_range(8000..65000));
        config
            .set_listeners(vec![locator.parse().unwrap()])
            .unwrap();
        let session = zenoh::open(config).wait().unwrap().into_arc();

        let expr = session
            .declare_expr(format!("/test/latency/zf/pong/{node_id})"))
            .wait()
            .unwrap();

        Ok(State::from(ScalPongSinkState {
            interval,
            nodes,
            msgs,
            session,
            expr,
            data: vec![],
        }))
    }

    fn finalize(&self, state: &mut State) -> ZFResult<()> {
        let _real_state = state.try_get::<ScalPongSinkState>()?;

        Ok(())
    }
}
