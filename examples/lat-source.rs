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

use async_std::sync::Arc;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use zenoh_flow::{default_output_rule, Node, Source};
use zenoh_flow::{types::ZFResult, zenoh_flow_derive::ZFState, Data, PortId, State, ZFState};
use zenoh_flow_perf::{get_epoch_us, LatData};

static SOURCE: &str = "Data";

#[derive(Debug)]
struct ThrSource;

#[derive(Debug, ZFState)]
struct ThrSourceState {
    pub data: Vec<u8>,
    pub interveal: f64,
}

#[async_trait]
impl Source for ThrSource {
    async fn run(&self, _context: &mut zenoh_flow::Context, state: &mut State) -> ZFResult<Data> {
        let real_state = state.try_get::<ThrSourceState>()?;

        async_std::task::sleep(Duration::from_secs_f64(real_state.interveal)).await;

        let ts = get_epoch_us();

        let data = LatData {
            data: real_state.data.clone(),
            ts,
        };
        Ok(Data::from::<LatData>(data))
    }
}

impl Node for ThrSource {
    fn initialize(&self, configuration: &Option<HashMap<String, String>>) -> State {
        let payload_size = match configuration {
            Some(conf) => conf.get("payload_size").unwrap().parse::<usize>().unwrap(),
            None => 8usize,
        };

        let interveal = match configuration {
            Some(conf) => conf.get("interveal").unwrap().parse::<f64>().unwrap(),
            None => 1f64,
        };

        let data = (0usize..payload_size)
            .map(|i| (i % 10) as u8)
            .collect::<Vec<u8>>();

        State::from(ThrSourceState { data, interveal })
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(ThrSource) as Arc<dyn Source>)
}
