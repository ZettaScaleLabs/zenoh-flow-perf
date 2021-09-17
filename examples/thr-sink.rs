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
use futures::future::{AbortHandle, Abortable};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::runtime::message::DataMessage;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{
    default_input_rule, downcast, downcast_mut, export_sink, types::ZFResult, Component, InputRule,
    PortId, State, Token,
};
use zenoh_flow::{Context, Sink};

struct ThrSink;

#[derive(ZFState, Debug, Clone)]
struct SinkState {
    pub payload_size: usize,
    pub accumulator: Arc<AtomicUsize>,
    pub abort_handle: AbortHandle,
}

#[async_trait]
impl Sink for ThrSink {
    async fn run(
        &self,
        _context: &mut Context,
        _state: &mut Box<dyn State>,
        _inputs: &mut HashMap<PortId, DataMessage>,
    ) -> ZFResult<()> {
        let state = downcast!(SinkState, _state).unwrap();
        state.accumulator.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl Component for ThrSink {
    fn initialize(&self, configuration: &Option<HashMap<String, String>>) -> Box<dyn State> {
        let payload_size = match configuration {
            Some(conf) => conf.get("payload_size").unwrap().parse::<usize>().unwrap(),
            None => 8usize,
        };

        let accumulator = Arc::new(AtomicUsize::new(0usize));

        let loop_accumulator = Arc::clone(&accumulator);
        let loop_payload_size = payload_size.clone();

        let print_loop = async move {
            println!("layer,scenario,test,name,size,messages");
            loop {
                let now = Instant::now();
                async_std::task::sleep(Duration::from_secs(1)).await;
                let elapsed = now.elapsed().as_micros() as f64;

                let c = loop_accumulator.swap(0, Ordering::Relaxed);
                if c > 0 {
                    let interval = 1_000_000.0 / elapsed;
                    println!(
                        "zenoh-flow,same-runtime,throughput,{},{},{}",
                        "test-name",
                        loop_payload_size,
                        (c as f64 / interval).floor() as usize
                    );
                }
            }
        };

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let _print_task = async_std::task::spawn(Abortable::new(print_loop, abort_registration));

        Box::new(SinkState {
            payload_size,
            accumulator,
            abort_handle,
        })
    }

    fn clean(&self, state: &mut Box<dyn State>) -> ZFResult<()> {
        let real_state = downcast_mut!(SinkState, state).unwrap();

        real_state.abort_handle.abort();
        Ok(())
    }
}

impl InputRule for ThrSink {
    fn input_rule(
        &self,
        _context: &mut Context,
        state: &mut Box<dyn State>,
        tokens: &mut HashMap<PortId, Token>,
    ) -> ZFResult<bool> {
        default_input_rule(state, tokens)
    }
}

export_sink!(register);

fn register() -> ZFResult<Arc<dyn Sink>> {
    Ok(Arc::new(ThrSink) as Arc<dyn Sink>)
}
