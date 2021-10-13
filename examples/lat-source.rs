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
use zenoh_flow::{
    downcast, types::ZFResult, zenoh_flow_derive::ZFState, zf_data, SerDeData, State,
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
    async fn run(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut Box<dyn zenoh_flow::State>,
    ) -> ZFResult<SerDeData> {
        let real_state = downcast!(ThrSourceState, state).unwrap();

        async_std::task::sleep(Duration::from_secs_f64(real_state.interveal)).await;

        let ts = get_epoch_us();

        let data = LatData {
            data: real_state.data.clone(),
            ts,
        };

        Ok(zf_data!(data))
    }
}

impl Node for ThrSource {
    fn initialize(
        &self,
        configuration: &Option<HashMap<String, String>>,
    ) -> Box<dyn zenoh_flow::State> {
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

        Box::new(ThrSourceState { data, interveal })
    }

    fn clean(&self, _state: &mut Box<dyn State>) -> ZFResult<()> {
        Ok(())
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(ThrSource) as Arc<dyn Source>)
}
