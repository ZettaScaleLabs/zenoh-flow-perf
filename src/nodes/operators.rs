use crate::nodes::{LAT_PORT, THR_PORT};
use crate::{get_epoch_us, Latency};
use async_trait::async_trait;
use std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{AsyncIteration, Configuration, Data, Inputs, Message, Node,
    Operator, Outputs, ZFResult,
};



// Latency OPERATOR

#[derive(Debug)]
pub struct NoOp;

#[async_trait]
impl Operator for NoOp {
    async fn setup(
        &self,
        _configuration: &Option<Configuration>,
        inputs: Inputs,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
        let input = inputs.get(LAT_PORT).unwrap()[0].clone();
        let output = outputs.get(LAT_PORT).unwrap()[0].clone();

        Arc::new(async move || {
            if let Ok((_, msg)) = input.recv().await {
                match msg {
                    Message::Data(mut msg) => {
                        output.send(msg.get_inner_data().clone(), None).await.unwrap();
                    }
                    _ => (),
                }
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
        _outputs: Outputs,
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
                match msg {
                    Message::Data(mut msg) => {
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
        _configuration: &Option<Configuration>,
        inputs: Inputs,
        outputs: Outputs,
    ) -> Arc<dyn AsyncIteration> {
        let input = inputs.get(THR_PORT).unwrap()[0].clone();
        let output = outputs.get(THR_PORT).unwrap()[0].clone();

        Arc::new(async move || {
            if let Ok((_, msg)) = input.recv().await {
                match msg {
                    Message::Data(mut msg) => {
                        output.send(msg.get_inner_data().clone(), None).await.unwrap();
                    }
                    _ => (),
                }
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

        let _state = IROpState { _inputs: op_inputs };

        let input = inputs.get("Data0").unwrap()[0].clone();
        let output = outputs.get(LAT_PORT).unwrap()[0].clone();

        Arc::new(async move || {
            if let Ok((_, msg)) = input.recv().await {
                match msg {
                    Message::Data(mut msg) => {
                        let data = msg.get_inner_data().try_get::<Latency>()?;
                        let now = get_epoch_us();

                        let elapsed = now - data.ts;
                        let data = Data::from(Latency { ts: elapsed });
                        output.send(data, None).await.unwrap();
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
