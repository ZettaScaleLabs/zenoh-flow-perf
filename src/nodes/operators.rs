//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use crate::nodes::{LAT_PORT, THR_PORT};
use crate::{get_epoch_us, Latency};
use async_trait::async_trait;
use std::sync::Arc;
use zenoh_flow::prelude::*;

// Latency OPERATOR

#[derive(Debug)]
pub struct NoOp;

#[async_trait]
impl Operator for NoOp {
    async fn setup(
        &self,
        _ctx: &mut Context,
        _configuration: &Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let input = inputs.take_into_arc(LAT_PORT).unwrap();
        let output = outputs.take_into_arc(LAT_PORT).unwrap();

        Ok(Some(Box::new(move || {
            let c_input = input.clone();
            let c_output = output.clone();

            async move {
                if let Ok(Message::Data(mut msg)) = c_input.recv_async().await {
                    c_output
                        .send_async(msg.get_inner_data().clone(), None)
                        .await
                        .unwrap();
                }
                Ok(())
            }
        })))
    }
}

// OPERATOR

#[derive(Debug, Clone)]
struct LatOpState {
    pipeline: u64,
    _interval: f64,
    msgs: u64,
    layer: String,
}

#[derive(Debug)]
pub struct NoOpPrint;

#[async_trait]
impl Operator for NoOpPrint {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
        mut _outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
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

        let multi = match configuration {
            Some(conf) => conf["multi"].as_bool().unwrap(),
            None => false,
        };

        let layer = match multi {
            true => "zf-src-op-multi".to_string(),
            false => "zf-src-op".to_string(),
        };

        let state = Arc::new(LatOpState {
            _interval: interval,
            pipeline,
            msgs,
            layer,
        });

        let input = inputs.take_into_arc(LAT_PORT).unwrap();

        Ok(Some(Box::new(move || {
            let c_input = input.clone();
            let c_state = state.clone();

            async move {
                if let Ok(Message::Data(mut msg)) = c_input.recv_async().await {
                    let data = msg.get_inner_data().try_get::<Latency>()?;
                    let now = get_epoch_us();

                    let elapsed = now - data.ts;
                    let msgs = c_state.msgs;
                    let pipeline = c_state.pipeline;
                    let layer = &c_state.layer;
                    println!("{layer},scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");
                }
                Ok(())
            }
        })))
    }
}

// THR OPERATOR

#[derive(Debug)]
pub struct ThrNoOp;
#[async_trait]
impl Operator for ThrNoOp {
    async fn setup(
        &self,
        _ctx: &mut Context,
        _configuration: &Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let input = inputs.take_into_arc(THR_PORT).unwrap();
        let output = outputs.take_into_arc(THR_PORT).unwrap();

        Ok(Some(Box::new(move || {
            let c_input = input.clone();
            let c_output = output.clone();

            async move {
                if let Ok(Message::Data(mut msg)) = c_input.recv_async().await {
                    c_output
                        .send_async(msg.get_inner_data().clone(), None)
                        .await
                        .unwrap();
                }
                Ok(())
            }
        })))
    }
}

#[derive(Debug, Clone)]
struct IROpState {
    _inputs: u64,
}

// OPERATOR

#[derive(Debug)]
pub struct IRNoOp;
#[async_trait]
impl Operator for IRNoOp {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let op_inputs = match configuration {
            Some(conf) => conf["inputs"].as_u64().unwrap(),
            None => 1,
        };

        let _state = IROpState { _inputs: op_inputs };

        let input = inputs.take_into_arc("Data0").unwrap();
        let output = outputs.take_into_arc(LAT_PORT).unwrap();
        Ok(Some(Box::new(move || {
            let c_input = input.clone();
            let c_output = output.clone();

            async move {
                if let Ok(Message::Data(mut msg)) = c_input.recv_async().await {
                    let data = msg.get_inner_data().try_get::<Latency>()?;
                    let now = get_epoch_us();

                    let elapsed = now - data.ts;
                    let data = Data::from(Latency { ts: elapsed });
                    c_output.send_async(data, None).await.unwrap();
                }
                Ok(())
            }
        })))
    }
}
