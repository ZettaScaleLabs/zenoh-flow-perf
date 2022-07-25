use crate::nodes::{LAT_PORT, THR_PORT};
use crate::{get_epoch_us, Latency};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{
    zf_empty_state, AsyncIteration, Configuration, Data, Inputs, Message, Node, NodeOutput,
    Operator, Outputs, PortId, ZFResult,
};

use std::convert::TryFrom;
use uhlc::Timestamp;
use uhlc::ID;

// Latency OPERATOR

#[derive(Debug)]
pub struct NoOp;

#[async_trait]
impl Operator for NoOp {
    async fn setup(
        &self,
        configuration: &Option<Configuration>,
        inputs: Inputs,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
        let input = inputs.get(LAT_PORT).unwrap()[0].clone();
        let output = outputs.get(LAT_PORT).unwrap()[0].clone();

        Arc::new(async move || {
            if let Ok((_, msg)) = input.recv().await {
                output.send(msg).await.unwrap();
            }
            Ok(())
        })
    }
}

#[async_trait]
impl Node for NoOp {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

// OPERATOR

#[derive(ZFState, Debug, Clone)]
struct LatOpState {
    pipeline: u64,
    interval: f64,
    msgs: u64,
    layer: String,
}

#[derive(Debug)]
pub struct NoOpPrint;

#[async_trait]
impl Operator for NoOpPrint {
    async fn setup(
        &self,
        configuration: &Option<Configuration>,
        inputs: Inputs,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
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

        let state = LatOpState {
            interval,
            pipeline,
            msgs,
            layer,
        };

        let input = inputs.get(LAT_PORT).unwrap()[0].clone();

        Arc::new(async move || {
            if let Ok((_, msg)) = input.recv().await {
                match msg.as_ref() {
                    Message::Data(msg) => {
                        let mut msg = msg.clone();
                        let data = msg.get_inner_data().try_get::<Latency>()?;
                        let now = get_epoch_us();

                        let elapsed = now - data.ts;
                        let msgs = state.msgs;
                        let pipeline = state.pipeline;
                        let layer = state.layer;
                        println!(
                            "{layer},scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us"
                        );
                    }
                    _ => (),
                }
            }
            Ok(())
        })
    }
}

#[async_trait]
impl Node for NoOpPrint {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

// THR OPERATOR

#[derive(Debug)]
pub struct ThrNoOp;
#[async_trait]
impl Operator for ThrNoOp {
    async fn setup(
        &self,
        configuration: &Option<Configuration>,
        inputs: Inputs,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
        let input = inputs.get(THR_PORT).unwrap()[0].clone();
        let output = outputs.get(THR_PORT).unwrap()[0].clone();

        Arc::new(async move || {
            if let Ok((_, msg)) = input.recv().await {
                output.send(msg).await.unwrap();
            }
            Ok(())
        })
    }
}

#[async_trait]
impl Node for ThrNoOp {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

#[derive(ZFState, Debug, Clone)]
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
        configuration: &Option<Configuration>,
        inputs: Inputs,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
        let op_inputs = match configuration {
            Some(conf) => conf["inputs"].as_u64().unwrap(),
            None => 1,
        };

        let state = IROpState { _inputs: op_inputs };

        let input = inputs.get("Data0").unwrap()[0].clone();
        let output = outputs.get(LAT_PORT).unwrap()[0].clone();
        let buf = [0x00, 0x00];
        let id = ID::try_from(&buf[..1]).unwrap();
        let ts = Timestamp::new(uhlc::NTP64(0), id);

        Arc::new(async move || {
            if let Ok((_, msg)) = input.recv().await {
                match msg.as_ref() {
                    Message::Data(msg) => {
                        let mut msg = msg.clone();
                        let data = msg.get_inner_data().try_get::<Latency>()?;
                        let now = get_epoch_us();

                        let elapsed = now - data.ts;
                        let data = Data::from(Latency { ts: elapsed });
                        let msg = Message::from_serdedata(data, ts);
                        output.send(Arc::new(msg)).await.unwrap();
                    }
                    _ => (),
                }
            }
            Ok(())
        })
    }
}

#[async_trait]
impl Node for IRNoOp {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}
