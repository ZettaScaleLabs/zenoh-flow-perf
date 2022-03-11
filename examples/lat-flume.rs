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
use zenoh_flow::async_std::task;
use zenoh_flow_perf::{get_epoch_us, Latency};

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";

#[derive(Parser, Debug)]
struct CallArgs {
    /// Config file
    #[clap(short, long, default_value = DEFAULT_PIPELINE)]
    pipeline: u64,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
}

#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::parse();

    let (sender_ping, receiver_ping) = flume::unbounded::<Latency>();
    let (sender_pong, receiver_pong) = flume::unbounded::<()>();

    let c_msgs = args.msgs.clone();
    let pipeline_msgs = args.pipeline.clone();
    task::spawn(async move {
        while let Ok(data) = receiver_ping.recv_async().await {
            let now = get_epoch_us();
            let elapsed = now - data.ts;

            // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
            println!("flume,scenario-name,latency,pipeline,{c_msgs},{pipeline_msgs},{elapsed},us");
            sender_pong.send_async(()).await.unwrap();
        }
    });

    let interval = 1.0 / (args.msgs as f64);

    loop {
        task::sleep(Duration::from_secs_f64(interval)).await;
        let msg = Latency { ts: get_epoch_us() };
        sender_ping.send_async(msg).await.unwrap();
        let _ = receiver_pong.recv_async().await.unwrap();
    }
}
