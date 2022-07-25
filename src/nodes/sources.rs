use crate::{get_epoch_us, Latency, ThrData};

use async_trait::async_trait;
use std::convert::TryFrom;
use std::time::Duration;
use uhlc::Timestamp;
use uhlc::ID;
use zenoh::prelude::*;
use zenoh::subscriber::Subscriber;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{AsyncIteration, Configuration, Data, Message, Node, Outputs, Source, ZFResult};

use super::{LAT_PORT, THR_PORT};

// Latency SOURCE
#[derive(Debug)]
pub struct LatSource;

#[derive(Debug, ZFState, Clone)]
struct LatSourceState {
    interval: f64,
}

#[async_trait]
impl Source for LatSource {
    async fn setup(
        &self,
        configuration: &Option<Configuration>,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let state = LatSourceState { interval };
        let output = outputs.get(LAT_PORT).unwrap()[0].clone();
        let buf = [0x00, 0x00];
        let id = ID::try_from(&buf[..1]).unwrap();

        Arc::new(async move || {
            async_std::task::sleep(Duration::from_secs_f64(state.interval)).await;
            let data = Data::from(Latency { ts: get_epoch_us() });
            let ts = Timestamp::new(uhlc::NTP64(0), id);
            let msg = Message::from_serdedata(data, ts);
            output.send(Arc::new(msg)).await.unwrap();
            Ok(())
        })
    }
}

#[async_trait]
impl Node for LatSource {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

// Ping SOURCE

#[derive(Debug)]
pub struct PingSource;

#[derive(Debug, ZFState, Clone)]
struct PingSourceState {
    interval: f64,
    sub: Arc<Subscriber<'static>>,
    first: bool,
}

impl PingSourceState {
    fn new(interval: f64, sub: Subscriber<'static>) -> Self {
        Self {
            interval,
            sub: Arc::new(sub),
            first: true,
        }
    }
}

#[async_trait]
impl Source for PingSource {
    async fn setup(
        &self,
        configuration: &Option<Configuration>,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
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

        let mut state = PingSourceState::new(interval, sub);

        let output = outputs.get(LAT_PORT).unwrap()[0].clone();
        let buf = [0x00, 0x00];
        let id = ID::try_from(&buf[..1]).unwrap();

        Arc::new(async move || {
            async_std::task::sleep(Duration::from_secs_f64(state.interval)).await;
            if !state.first {
                let _ = state.sub.recv();
            } else {
                state.first = false;
            }

            let data = Data::from(Latency { ts: get_epoch_us() });
            let ts = Timestamp::new(uhlc::NTP64(0), id);
            let msg = Message::from_serdedata(data, ts);
            output.send(Arc::new(msg)).await.unwrap();
            Ok(())
        })
    }
}

#[async_trait]
impl Node for PingSource {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

// THR SOURCE

#[derive(Debug)]
pub struct ThrSource;

#[derive(Debug, ZFState, Clone)]
struct ThrSourceState {
    pub data: Arc<ThrData>,
}

#[async_trait]
impl Source for ThrSource {
    async fn setup(
        &self,
        configuration: &Option<Configuration>,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let data = Arc::new(ThrData {
            data: (0usize..payload_size)
                .map(|i| (i % 10) as u8)
                .collect::<Vec<u8>>(),
        });

        let state = ThrSourceState { data };
        let output = outputs.get(THR_PORT).unwrap()[0].clone();
        let buf = [0x00, 0x00];
        let id = ID::try_from(&buf[..1]).unwrap();

        Arc::new(async move || {
            let data = state.data.clone();
            let data = Data::from_arc::<ThrData>(data);
            let ts = Timestamp::new(uhlc::NTP64(0), id);
            let msg = Message::from_serdedata(data, ts);
            output.send(Arc::new(msg)).await.unwrap();
            Ok(())
        })
    }
}

#[async_trait]
impl Node for ThrSource {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

// Scalability Ping SOURCE

#[derive(Debug)]
pub struct ScalPingSource;

#[derive(Debug, ZFState, Clone)]
struct ScalPingSourceState {
    interval: f64,
    subs: Arc<Vec<Subscriber<'static>>>,
    first: bool,
}

impl ScalPingSourceState {
    fn new(interval: f64, subs: Vec<Subscriber<'static>>) -> Self {
        Self {
            interval,
            subs: Arc::new(subs),
            first: true,
        }
    }
}

#[async_trait]
impl Source for ScalPingSource {
    async fn setup(
        &self,
        configuration: &Option<Configuration>,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
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
                .declare_expr("/test/latency/zf/pong/0")
                .wait()
                .unwrap();
            let sub = session.subscribe(key_expr_pong).wait().unwrap();
            subs.push(sub);
        }

        let mut state = ScalPingSourceState::new(interval, subs);

        let output = outputs.get(LAT_PORT).unwrap()[0].clone();
        let buf = [0x00, 0x00];
        let id = ID::try_from(&buf[..1]).unwrap();

        Arc::new(async move || {
            async_std::task::sleep(Duration::from_secs_f64(state.interval)).await;
            async_std::task::sleep(Duration::from_secs_f64(state.interval)).await;
            if !state.first {
                for sub in &*state.subs {
                    let _ = sub.recv();
                }
            } else {
                state.first = false;
            }

            let data = Data::from(Latency { ts: get_epoch_us() });
            let ts = Timestamp::new(uhlc::NTP64(0), id);
            let msg = Message::from_serdedata(data, ts);
            output.send(Arc::new(msg)).await.unwrap();
            Ok(())
        })
    }
}

#[async_trait]
impl Node for ScalPingSource {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}
