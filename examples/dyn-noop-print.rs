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

use std::collections::HashMap;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{
    default_input_rule, default_output_rule, export_operator, types::ZFResult, Configuration, Data,
    LocalDeadlineMiss, Node, NodeOutput, Operator, PortId, State,
};
use zenoh_flow_perf::{get_epoch_us, Latency};
static PORT: &str = "Data";

#[derive(ZFState, Debug, Clone)]
struct LatOpState {
    pipeline: u64,
    interval: f64,
    msgs: u64,
}

// OPERATOR

#[derive(Debug)]
struct NoOp;

impl Operator for NoOp {
    fn input_rule(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut State,
        tokens: &mut HashMap<PortId, zenoh_flow::InputToken>,
    ) -> zenoh_flow::ZFResult<bool> {
        default_input_rule(state, tokens)
    }

    fn run(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut State,
        inputs: &mut HashMap<PortId, zenoh_flow::runtime::message::DataMessage>,
    ) -> zenoh_flow::ZFResult<HashMap<zenoh_flow::PortId, Data>> {
        let results: HashMap<PortId, Data> = HashMap::new();

        let real_state = state.try_get::<LatOpState>()?;
        let _ = real_state.interval;

        let data = inputs
            .get_mut(PORT)
            .unwrap()
            .get_inner_data()
            .try_get::<Latency>()?;

        let now = get_epoch_us();

        let elapsed = now - data.ts;
        let msgs = real_state.msgs;
        let pipeline = real_state.pipeline;
        // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
        println!("zf-source-op-multi,scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");

        Ok(results)
    }

    fn output_rule(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut State,
        outputs: HashMap<PortId, Data>,
        _deadline_miss: Option<LocalDeadlineMiss>,
    ) -> zenoh_flow::ZFResult<HashMap<zenoh_flow::PortId, NodeOutput>> {
        default_output_rule(state, outputs)
    }
}

impl Node for NoOp {
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

        Ok(State::from(LatOpState {
            interval,
            pipeline,
            msgs,
        }))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

export_operator!(register);

fn register() -> ZFResult<Arc<dyn Operator>> {
    Ok(Arc::new(NoOp) as Arc<dyn Operator>)
}
