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

use crate::{get_epoch_us, Latency, ThrData};

use async_std::sync::Mutex;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use zenoh::prelude::*;
use zenoh::subscriber::Subscriber;
use zenoh_flow::prelude::*;

use super::{LAT_PORT, THR_PORT};

// Latency SOURCE
#[derive(Debug)]
pub struct LatSource;

#[derive(Debug, Clone)]
struct LatSourceState {
    interval: f64,
}

#[async_trait]
impl Source for LatSource {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let state = Arc::new(LatSourceState { interval });
        let output = outputs.take_into_arc(LAT_PORT).unwrap();

        Ok(Some(Box::new(move || {
            let c_state = state.clone();
            let c_output = output.clone();

            async move {
                async_std::task::sleep(Duration::from_secs_f64(c_state.interval)).await;
                let data = Data::from(Latency { ts: get_epoch_us() });
                c_output.send_async(data, None).await
            }
        })))
    }
}

// Ping SOURCE

#[derive(Debug)]
pub struct PingSource;

#[derive(Debug, Clone)]
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
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
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

        let state = Arc::new(Mutex::new(PingSourceState::new(interval, sub)));

        let output = outputs.take_into_arc(LAT_PORT).unwrap();

        Ok(Some(Box::new(move || {
            let c_output = output.clone();
            let c_state = state.clone();

            async move {
                let mut c_state = c_state.lock().await;
                async_std::task::sleep(Duration::from_secs_f64(c_state.interval)).await;
                if !c_state.first {
                    let _ = c_state.sub.recv();
                } else {
                    c_state.first = false;
                }

                let data = Data::from(Latency { ts: get_epoch_us() });
                c_output.send_async(data, None).await
            }
        })))
    }
}

// THR SOURCE

#[derive(Debug)]
pub struct ThrSource;

#[derive(Debug, Clone)]
struct ThrSourceState {
    pub data: Arc<ThrData>,
}

#[async_trait]
impl Source for ThrSource {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let data = Arc::new(ThrData {
            data: (0usize..payload_size)
                .map(|i| (i % 10) as u8)
                .collect::<Vec<u8>>(),
        });

        let state = Arc::new(ThrSourceState { data });
        let output = outputs.take_into_arc(THR_PORT).unwrap();

        Ok(Some(Box::new(move || {
            let c_output = output.clone();
            let c_state = state.clone();
            async move {
                let data = c_state.data.clone();
                let data = Data::from(data);
                c_output.send_async(data, None).await
            }
        })))
    }
}

// Scalability Ping SOURCE

#[derive(Debug)]
pub struct ScalPingSource;

#[derive(Debug, Clone)]
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
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
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

        let state = Arc::new(Mutex::new(ScalPingSourceState::new(interval, subs)));

        let output = outputs.take_into_arc(LAT_PORT).unwrap();

        Ok(Some(Box::new(move || {
            let c_state = state.clone();
            let c_output = output.clone();

            async move {
                let mut c_state = c_state.lock().await;
                async_std::task::sleep(Duration::from_secs_f64(c_state.interval)).await;
                if !c_state.first {
                    for sub in &*c_state.subs {
                        let _ = sub.recv();
                    }
                } else {
                    c_state.first = false;
                }

                let data = Data::from(Latency { ts: get_epoch_us() });
                c_output.send_async(data, None).await
            }
        })))
    }
}
