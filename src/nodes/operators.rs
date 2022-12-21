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

/*
 ***************************************************************************************************
 *
 * LATENCY OPERATOR
 *
 ***************************************************************************************************
 */

pub struct NoOp {
    input: InputRaw,
    output: OutputRaw,
}

#[async_trait::async_trait]
impl Node for NoOp {
    async fn iteration(&self) -> Result<()> {
        self.output.forward(self.input.recv().await?).await
    }
}

#[async_trait::async_trait]
impl Operator for NoOp {
    async fn new(
        _context: Context,
        _configuration: Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<Self> {
        Ok(Self {
            input: inputs.take_raw(LAT_PORT).unwrap(),
            output: outputs.take_raw(LAT_PORT).unwrap(),
        })
    }
}

/*
 ***************************************************************************************************
 *
 * NO OPERATOR PRINT
 *
 ***************************************************************************************************
 */

#[derive(Debug, Clone)]
struct LatOpState {
    pipeline: u64,
    _interval: f64,
    msgs: u64,
    layer: String,
}

#[derive(Debug)]
pub struct NoOpPrint {
    input: Input<Latency>,
    state: Arc<LatOpState>,
}

#[async_trait]
impl Node for NoOpPrint {
    async fn iteration(&self) -> Result<()> {
        if let Ok((Message::Data(msg), _)) = self.input.recv().await {
            let now = get_epoch_us();

            let elapsed = now - (*msg).ts;
            let msgs = self.state.msgs;
            let pipeline = self.state.pipeline;
            let layer = &self.state.layer;
            println!("{layer},scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");
        }
        Ok(())
    }
}

pub struct NoOpPrintFactory;

#[async_trait]
impl Operator for NoOpPrint {
    async fn new(
        _ctx: Context,
        configuration: Option<Configuration>,
        mut inputs: Inputs,
        mut _outputs: Outputs,
    ) -> Result<Self> {
        let interval = match &configuration {
            Some(conf) => conf["interval"].as_f64().unwrap(),
            None => 1.0f64,
        };

        let pipeline = match &configuration {
            Some(conf) => conf["pipeline"].as_u64().unwrap(),
            None => 1u64,
        };

        let msgs = match &configuration {
            Some(conf) => conf["msgs"].as_u64().unwrap(),
            None => 1u64,
        };

        let multi = match &configuration {
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

        let input = inputs.take(LAT_PORT).unwrap();

        Ok(Self { state, input })
    }
}

/*
 ***************************************************************************************************
 *
 * THROUGHPUT OPERATOR
 *
 ***************************************************************************************************
 */

pub struct ThrNoOp {
    input: InputRaw,
    output: OutputRaw,
}

#[async_trait]
impl Node for ThrNoOp {
    async fn iteration(&self) -> Result<()> {
        self.output.forward(self.input.recv().await?).await
    }
}

#[async_trait]
impl Operator for ThrNoOp {
    async fn new(
        _ctx: Context,
        _configuration: Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<Self> {
        let input = inputs.take_raw(THR_PORT).unwrap();
        let output = outputs.take_raw(THR_PORT).unwrap();

        Ok(Self { input, output })
    }
}

/*
 ***************************************************************************************************
 *
 * LAST OPERATOR FOR SCALABILITY STATIC
 *
 ***************************************************************************************************
 */

pub struct ScalNoOp {
    inputs: Vec<Input<Latency>>,
    output: Output<Latency>,
}

#[async_trait]
impl Node for ScalNoOp {
    async fn iteration(&self) -> Result<()> {
        let fut_inputs: Vec<_> = self.inputs.iter().map(|input| input.recv()).collect();
        let data = futures::future::try_join_all(fut_inputs)
            .await?
            // Get the first one from the list --- we know it’s the first one because `try_join_all`
            // preserves the order.
            .pop()
            .expect("ScalNoOp received no data");

        match data {
            (Message::Data(latency), _) => {
                let now = get_epoch_us();
                let elapsed = now - (*latency).ts;
                self.output.send(Latency { ts: elapsed }, None).await
            }
            _ => panic!("Unimplemented"),
        }
    }
}

#[async_trait]
impl Operator for ScalNoOp {
    async fn new(
        _ctx: Context,
        configuration: Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<Self> {
        let ins: Vec<Input<Latency>> = match &configuration {
            Some(configuration) => {
                let port_ids = configuration["port_ids"]
                    .as_array()
                    .expect("port_ids should be an array");
                port_ids
                    .iter()
                    .map(|val| {
                        let port_id = val.as_str().expect("value should be a string");
                        match inputs.take(port_id) {
                            Some(input) => input,
                            None => panic!("Input < {} > not found", port_id),
                        }
                    })
                    .collect()
            }
            None => panic!("The port_ids of the inputs must be specified for this operator"),
        };

        Ok(ScalNoOp {
            inputs: ins,
            output: outputs
                .take(LAT_PORT)
                .expect("Output LAT_PORT should exist"),
        })
    }
}
