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

use structopt::StructOpt;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::async_std::task;

static DEFAULT_INT: &str = "1";
static DEFAULT_SIZE: &str = "8";
static DEFAULT_DURATION: &str = "60";
use std::time::{Duration, Instant};

#[derive(StructOpt, Debug)]
struct CallArgs {
    /// Config file
    #[structopt(short, long, default_value = DEFAULT_SIZE)]
    size: u64,
    #[structopt(short, long, default_value = DEFAULT_INT)]
    interveal: f64,
    #[structopt(short, long, default_value = DEFAULT_DURATION)]
    duration: u64,
}

#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::from_args();

    let (sender, receiver) = flume::unbounded::<(Instant, Arc<Vec<u8>>)>();

    task::spawn(async move {
        while let Ok(data) = receiver.recv_async().await {
            let elapsed = data.0.elapsed().as_nanos() as u64;

            println!(
                "flume,scenario-name,latency,test-name,{},y,x,{}",
                data.1.len(),
                elapsed
            );
        }
    });

    let d = args.duration;
    task::spawn(async move {
        task::sleep(Duration::from_secs(d)).await;
        std::process::exit(0);
    });

    let data = Arc::new(vec![0; args.size as usize]);

    loop {
        task::sleep(Duration::from_secs_f64(args.interveal)).await;
        let now_s = Instant::now();
        let d = (now_s, Arc::clone(&data));
        sender.send_async(d).await.unwrap();
    }
}
