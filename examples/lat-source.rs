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
use std::time::Duration;
use zenoh_flow::{
    types::ZFResult, zenoh_flow_derive::ZFState, Configuration, Data, State,
};
use zenoh_flow::{Node, Source};
use zenoh_flow_perf::{get_epoch_us, LatData};

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
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let payload_size = match configuration {
            Some(conf) => conf["payload_size"].as_u64().unwrap() as usize,
            None => 8usize,
        };

        let interveal = match configuration {
            Some(conf) => conf["interveal"].as_f64().unwrap(),
            None => 1f64,
        };

        let data = (0usize..payload_size)
            .map(|i| (i % 10) as u8)
            .collect::<Vec<u8>>();

        Ok(State::from(ThrSourceState { data, interveal }))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(ThrSource) as Arc<dyn Source>)
}
