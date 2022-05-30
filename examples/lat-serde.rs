//
// Copyright (c) 2017, 2022 ZettaScale Technology.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale zenoh team, <zenoh@zettascale.tech>
//

use clap::Parser;
use std::time::Duration;
use uhlc::HLC;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::{Data, Message};
use zenoh_flow_perf::{get_epoch_us, Latency};

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";

#[derive(Parser, Debug)]
struct CallArgs {
    #[clap(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u64,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
}

fn noop(input: Vec<u8>, hlc: Arc<HLC>) -> Vec<u8> {
    let deserialized_zf: Message = bincode::deserialize(&input).unwrap();
    match deserialized_zf {
        Message::Data(mut data) => {
            let inner_data = data.get_inner_data().clone();
            let zf_msg = Message::from_serdedata(inner_data, hlc.new_timestamp(), vec![], vec![]);
            zf_msg.serialize_bincode().unwrap()
        }
        _ => panic!("Should never enter here!"),
    }
}

fn pipeline(size: u64, input: Vec<u8>, hlc: Arc<HLC>) -> Vec<u8> {
    let mut in_data = input;
    for _ in 0..size {
        in_data = noop(in_data, hlc.clone());
    }
    in_data
}

#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::parse();
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());
    let interval = 1.0 / (args.msgs as f64);

    loop {
        async_std::task::sleep(Duration::from_secs_f64(interval)).await;
        let msg = Latency { ts: get_epoch_us() };
        let data = Data::from::<Latency>(msg);
        let zf_msg = Message::from_serdedata(data, hlc.new_timestamp(), vec![], vec![]);

        let serialized_zf = zf_msg.serialize_bincode().unwrap();

        let in_data = pipeline(args.pipeline, serialized_zf, hlc.clone());

        let deserialized_zf: Message = bincode::deserialize(&in_data).unwrap();

        // let deserialized_zf : Message = bincode::deserialize(&serialized_zf).unwrap();

        if let Message::Data(mut data) = deserialized_zf {
            let de_msg = data.get_inner_data().try_get::<Latency>().unwrap();
            let now = get_epoch_us();
            let elapsed = now - de_msg.ts;
            println!(
                "serde,scenario,latency,pipeline,{},{},{}",
                args.msgs, args.pipeline, elapsed
            );
        }
    }
}
