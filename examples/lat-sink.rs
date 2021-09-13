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
use std::collections::HashMap;
use zenoh_flow::async_std::sync::{Arc};
use zenoh_flow::runtime::message::ZFDataMessage;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{
    default_input_rule, get_input, export_sink, types::ZFResult, Token, ZFComponent,
    ZFComponentInputRule, ZFStateTrait,
};
use zenoh_flow::{ZFContext, ZFSinkTrait};
use zenoh_flow_perf::{LatData, get_epoch_us};

static INPUT: &str = "Data";

struct ThrSink;

#[derive(ZFState, Debug, Clone)]
struct SinkState {
    pub payload_size: usize,
}

#[async_trait]
impl ZFSinkTrait for ThrSink {
    async fn run(
        &self,
        _context: &mut ZFContext,
        _state: &mut Box<dyn ZFStateTrait>,
        inputs: &mut HashMap<String, ZFDataMessage>,
    ) -> ZFResult<()> {
        // let state = downcast!(SinkState, state).unwrap();
        let (_, data) = get_input!(LatData, String::from(INPUT), inputs)?;

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

impl ZFComponent for ThrSink {
    fn initialize(&self, configuration: &Option<HashMap<String, String>>) -> Box<dyn ZFStateTrait> {
        let payload_size = match configuration {
            Some(conf) => conf.get("payload_size").unwrap().parse::<usize>().unwrap(),
            None => 8usize,
        };

        Box::new(SinkState { payload_size })
    }

    fn clean(&self, _state: &mut Box<dyn ZFStateTrait>) -> ZFResult<()> {
        Ok(())
    }
}

impl ZFComponentInputRule for ThrSink {
    fn input_rule(
        &self,
        _context: &mut ZFContext,
        state: &mut Box<dyn ZFStateTrait>,
        tokens: &mut HashMap<String, Token>,
    ) -> ZFResult<bool> {
        default_input_rule(state, tokens)
    }
}

export_sink!(register);

fn register() -> ZFResult<Arc<dyn ZFSinkTrait>> {
    Ok(Arc::new(ThrSink) as Arc<dyn ZFSinkTrait>)
}
