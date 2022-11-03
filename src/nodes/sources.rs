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
use zenoh::{prelude::r#async::*, subscriber::FlumeSubscriber};
use zenoh_flow::prelude::*;

use super::{LAT_PORT, THR_PORT};

/*
 ***************************************************************************************************
 *
 *  LATENCY
 *
 ***************************************************************************************************
 */

pub struct LatSource {
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

pub struct LatSourceFactory;

#[async_trait::async_trait]
impl SourceFactoryTrait for LatSourceFactory {
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
        let output = outputs.take(LAT_PORT).expect("Missing output $LAT_PORT");

        Ok(Some(Arc::new(LatSource {
            sleep_interval,
            output,
        })))
    }
}

/*
 ***************************************************************************************************
 *
 *  PING
 *
 ***************************************************************************************************
 */

pub struct PingSourceFactory;

#[async_trait]
impl SourceFactoryTrait for PingSourceFactory {
    async fn new_source(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Arc<dyn Node>>> {
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let mut config = zenoh::config::Config::default();
        config
            .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
            .unwrap();
        let session = zenoh::open(config).res().await.unwrap().into_arc();

        let key_expr = session
            .declare_keyexpr("test/latency/zf/pong")
            .res()
            .await
            .unwrap();

        let sub = session.declare_subscriber(key_expr).res().await.unwrap();

        let state = Arc::new(Mutex::new(PingSourceState::new(interval, sub)));

        let output = outputs.take(LAT_PORT).unwrap();

        Ok(Some(Arc::new(PingSource { state, output })))
    }
}

pub struct PingSource<'a> {
    state: Arc<Mutex<PingSourceState<'a>>>,
    output: Output,
}

#[async_trait]
impl<'a> Node for PingSource<'a> {
    async fn iteration(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        async_std::task::sleep(Duration::from_secs_f64(state.interval)).await;
        if !state.first {
            let _ = state.sub.recv();
        } else {
            state.first = false;
        }

        let data = Data::from(Latency { ts: get_epoch_us() });
        self.output.send_async(data, None).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PingSourceState<'a> {
    interval: f64,
    sub: Arc<FlumeSubscriber<'a>>,
    first: bool,
}

impl<'a> PingSourceState<'a> {
    fn new(interval: f64, sub: FlumeSubscriber<'a>) -> Self {
        Self {
            interval,
            sub: Arc::new(sub),
            first: true,
        }
    }
}

/*
 ***************************************************************************************************
 *
 *  THROUGHPUT
 *
 ***************************************************************************************************
 */

pub struct ThrSourceFactory;

#[async_trait]
impl SourceFactoryTrait for ThrSourceFactory {
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

        let output = outputs.take(THR_PORT).unwrap();

        Ok(Some(Arc::new(ThrSource { data, output })))
    }
}

pub struct ThrSource {
    data: Arc<ThrData>,
    output: Output,
}

#[async_trait]
impl Node for ThrSource {
    async fn iteration(&self) -> Result<()> {
        self.output
            .send_async(Data::from(self.data.clone()), None)
            .await
    }
}

/*
 ***************************************************************************************************
 *
 *  SCALABILITY SOURCE
 *
 ***************************************************************************************************
 */

pub struct ScalPingSourceFactory;

#[async_trait]
impl SourceFactoryTrait for ScalPingSourceFactory {
    async fn new_source(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Arc<dyn Node>>> {
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
        let session = zenoh::open(config).res().await.unwrap().into_arc();

        let mut key_exp_vec = vec![];
        let mut subs = vec![];

        if mode == 2 {
            // 2 means fan out, 1 means fan in

            for i in 0..nodes {
                let key_expr_pong = session
                    .declare_keyexpr(format!("test/latency/zf/pong/{i}"))
                    .res()
                    .await
                    .unwrap();
                key_exp_vec.push(key_expr_pong);
            }

            for kx in &key_exp_vec {
                let sub = session.declare_subscriber(kx).res().await.unwrap();
                subs.push(sub);
            }
        } else {
            let key_expr_pong = session
                .declare_keyexpr("test/latency/zf/pong/0")
                .res()
                .await
                .unwrap();
            let sub = session
                .declare_subscriber(key_expr_pong)
                .res()
                .await
                .unwrap();
            subs.push(sub);
        }

        let state = Arc::new(Mutex::new(ScalPingSourceState::new(interval, subs)));

        let output = outputs.take(LAT_PORT).unwrap();

        Ok(Some(Arc::new(ScalPingSource { state, output })))
    }
}

pub struct ScalPingSource<'a> {
    state: Arc<Mutex<ScalPingSourceState<'a>>>,
    output: Output,
}

#[async_trait]
impl<'a> Node for ScalPingSource<'a> {
    async fn iteration(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        async_std::task::sleep(Duration::from_secs_f64(state.interval)).await;
        if !state.first {
            for sub in &*state.subs {
                let _ = sub.recv();
            }
        } else {
            state.first = false;
        }

        let data = Data::from(Latency { ts: get_epoch_us() });
        self.output.send_async(data, None).await
    }
}

#[derive(Debug, Clone)]
struct ScalPingSourceState<'a> {
    interval: f64,
    subs: Arc<Vec<FlumeSubscriber<'a>>>,
    first: bool,
}

impl<'a> ScalPingSourceState<'a> {
    fn new(interval: f64, subs: Vec<FlumeSubscriber<'a>>) -> Self {
        Self {
            interval,
            subs: Arc::new(subs),
            first: true,
        }
    }
}
