use crate::{get_epoch_us, Latency, ThrData};

use async_trait::async_trait;
use std::time::Duration;
use zenoh::prelude::*;
use zenoh::subscriber::Subscriber;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{Configuration, Context, Data, Node, Source, State, ZFResult};

// Latency SOURCE
#[derive(Debug)]
pub struct LatSource;

#[derive(Debug, ZFState)]
struct LatSourceState {
    interval: f64,
}

#[async_trait]
impl Source for LatSource {
    async fn run(&self, _context: &mut Context, state: &mut State) -> zenoh_flow::ZFResult<Data> {
        let real_state = state.try_get::<LatSourceState>()?;

        async_std::task::sleep(Duration::from_secs_f64(real_state.interval)).await;

        let msg = Latency { ts: get_epoch_us() };

        Ok(Data::from::<Latency>(msg))
    }
}

impl Node for LatSource {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        Ok(State::from(LatSourceState { interval }))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

// Ping SOURCE

#[derive(Debug)]
pub struct PingSource;

#[derive(Debug, ZFState)]
struct PingSourceState {
    interval: f64,
    sub: Subscriber<'static>,
    first: bool,
}

impl PingSourceState {
    fn new(interval: f64, sub: Subscriber<'static>) -> Self {
        Self {
            interval,
            sub,
            first: true,
        }
    }
}

#[async_trait]
impl Source for PingSource {
    async fn run(&self, _context: &mut Context, state: &mut State) -> zenoh_flow::ZFResult<Data> {
        let mut real_state = state.try_get::<PingSourceState>()?;

        async_std::task::sleep(Duration::from_secs_f64(real_state.interval)).await;
        if !real_state.first {
            let _ = real_state.sub.recv();
        } else {
            real_state.first = false;
        }

        let msg = Latency { ts: get_epoch_us() };

        Ok(Data::from::<Latency>(msg))
    }
}

impl Node for PingSource {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let mut config = zenoh::config::Config::default();
        config
            .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
            .unwrap();
        let session = zenoh::open(config).wait().unwrap().into_arc();

        let key_expr_pong = session
            .declare_expr("/test/latency/zf/pong")
            .wait()
            .unwrap();

        let sub = session.subscribe(&key_expr_pong).wait().unwrap();

        Ok(State::from(PingSourceState::new(interval, sub)))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

// THR SOURCE

#[derive(Debug)]
pub struct ThrSource;

#[derive(Debug, ZFState)]
struct ThrSourceState {
    pub data: Arc<ThrData>,
}

#[async_trait]
impl Source for ThrSource {
    async fn run(&self, _context: &mut Context, state: &mut State) -> zenoh_flow::ZFResult<Data> {
        let real_state = state.try_get::<ThrSourceState>()?;

        let data = real_state.data.clone();

        Ok(Data::from_arc::<ThrData>(data))
    }
}

impl Node for ThrSource {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let data = Arc::new(ThrData {
            data: (0usize..payload_size)
                .map(|i| (i % 10) as u8)
                .collect::<Vec<u8>>(),
        });

        Ok(State::from(ThrSourceState { data }))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

// Scalability Ping SOURCE

#[derive(Debug)]
pub struct ScalPingSource;

#[derive(Debug, ZFState)]
struct ScalPingSourceState {
    interval: f64,
    subs: Vec<Subscriber<'static>>,
    first: bool,
}

impl ScalPingSourceState {
    fn new(interval: f64, subs: Vec<Subscriber<'static>>) -> Self {
        Self {
            interval,
            subs,
            first: true,
        }
    }
}

#[async_trait]
impl Source for ScalPingSource {
    async fn run(&self, _context: &mut Context, state: &mut State) -> zenoh_flow::ZFResult<Data> {
        let mut real_state = state.try_get::<ScalPingSourceState>()?;

        async_std::task::sleep(Duration::from_secs_f64(real_state.interval)).await;
        if !real_state.first {
            for sub in &real_state.subs {
                let _ = sub.recv();
            }
        } else {
            real_state.first = false;
        }

        let msg = Latency { ts: get_epoch_us() };

        Ok(Data::from::<Latency>(msg))
    }
}

impl Node for ScalPingSource {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let nodes = match configuration {
            Some(conf) => conf["nodes"].as_u64().unwrap(),
            None => 1u64,
        };

        let mode = match configuration {
            Some(conf) => conf["mode"].as_u64().unwrap(),
            None => 1u64,
        };

        let mut config = zenoh::config::Config::default();
        config
            .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
            .unwrap();
        let session = zenoh::open(config).wait().unwrap().into_arc();

        let mut key_exp_vec = vec![];
        let mut subs = vec![];
        if mode == 2 {
            // 2 means fan out, 1 means fan in

            for i in 0..nodes {
                let key_expr_pong = session
                    .declare_expr(format!("/test/latency/zf/pong/{i}"))
                    .wait()
                    .unwrap();
                key_exp_vec.push(key_expr_pong);
            }

            for kx in &key_exp_vec {
                let sub = session.subscribe(kx).wait().unwrap();
                subs.push(sub);
            }
        } else {
            let key_expr_pong = session
                .declare_expr(format!("/test/latency/zf/pong/0"))
                .wait()
                .unwrap();
            let sub = session.subscribe(key_expr_pong).wait().unwrap();
            subs.push(sub);
        }

        Ok(State::from(ScalPingSourceState::new(interval, subs)))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}
