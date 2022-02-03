//
// Copyright (c) 2017, 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//

use rand::Rng;
use async_trait::async_trait;
use std::time::Duration;
use zenoh::prelude::*;
use zenoh::subscriber::Subscriber;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{export_source, types::ZFResult, Configuration, Data, Node, State};
use zenoh_flow::{Context, Source};
use zenoh_flow_perf::{get_epoch_us, Latency};

// SOURCE

#[derive(Debug)]
struct PingSource;

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

        let mut rng = rand::thread_rng();
        let interval = match configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
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

export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(PingSource) as Arc<dyn Source>)
}
