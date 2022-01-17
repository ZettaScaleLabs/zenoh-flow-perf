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
use zenoh_flow::runtime::message::DataMessage;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{export_sink, types::ZFResult, Configuration, Node, State};
use zenoh_flow::{Context, Sink};
use zenoh_flow_perf::{get_epoch_us, LatData};

struct ThrSink;

#[derive(ZFState, Debug, Clone)]
struct SinkState {
    pub _payload_size: usize,
}

#[async_trait]
impl Sink for ThrSink {
    async fn run(
        &self,
        _context: &mut Context,
        _state: &mut State,
        mut input: DataMessage,
    ) -> ZFResult<()> {
        // let state = downcast!(SinkState, state).unwrap();

        let data = input.data.try_get::<LatData>()?;

        let now = get_epoch_us();

        let elapsed = now - data.ts;
        println!(
            "zenoh-flow,{},latency,{},{},{},{},{}",
            "scenario-name",
            "test-name",
            data.data.len(),
            "y",
            "x",
            elapsed
        );

        Ok(())
    }
}

impl Node for ThrSink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        Ok(State::from(SinkState {
            _payload_size: payload_size,
        }))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

export_sink!(register);

fn register() -> ZFResult<Arc<dyn Sink>> {
    Ok(Arc::new(ThrSink) as Arc<dyn Sink>)
}
