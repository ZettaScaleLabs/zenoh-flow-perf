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

use async_trait::async_trait;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{export_sink, types::ZFResult, Configuration, Node, State};
use zenoh_flow::{Context, Sink};
use zenoh_flow_perf::{get_epoch_us, Latency};

// SINK

struct LatSink;

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
        println!("zenoh-flow-multi,scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");

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

export_sink!(register);

fn register() -> ZFResult<Arc<dyn Sink>> {
    Ok(Arc::new(LatSink) as Arc<dyn Sink>)
}
