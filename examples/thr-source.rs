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
use zenoh_flow::{default_output_rule, Node, Source};
use zenoh_flow::{
    downcast, types::ZFResult, zenoh_flow_derive::ZFState, PortId, Data, ZFState,
};
use zenoh_flow_perf::ThrData;

static SOURCE: &str = "Data";

#[derive(Debug)]
struct ThrSource;

#[derive(Debug, ZFState)]
struct ThrSourceState {
    pub data: Vec<u8>,
}

#[async_trait]
impl Source for ThrSource {
    async fn run(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut Box<dyn zenoh_flow::ZFState>,
    ) -> ZFResult<Data> {
        let real_state = downcast!(ThrSourceState, state).unwrap();

        let data = ThrData {
            data: real_state.data.clone(),
        };

        Ok(Data::from::<ThrData>(data))
    }
}

impl Node for ThrSource {
    fn initialize(
        &self,
        configuration: &Option<HashMap<String, String>>,
    ) -> Box<dyn zenoh_flow::ZFState> {
        let payload_size = match configuration {
            Some(conf) => conf.get("payload_size").unwrap().parse::<usize>().unwrap(),
            None => 8usize,
        };

        let data = (0usize..payload_size)
            .map(|i| (i % 10) as u8)
            .collect::<Vec<u8>>();

        Box::new(ThrSourceState { data })
    }

    fn clean(&self, _state: &mut Box<dyn ZFState>) -> ZFResult<()> {
        Ok(())
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(ThrSource) as Arc<dyn Source>)
}
