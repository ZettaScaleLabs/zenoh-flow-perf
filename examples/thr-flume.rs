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

use std::sync::atomic::{AtomicU64, Ordering};
use structopt::StructOpt;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::async_std::task;

static DEFAULT_INT: &str = "1";
static DEFAULT_SIZE: &str = "8";
static DEFAULT_DURATION: &str = "60";
use std::time::Duration;

#[derive(StructOpt, Debug)]
struct CallArgs {
    #[structopt(short, long, default_value = DEFAULT_SIZE)]
    size: u64,
    #[structopt(short, long, default_value = DEFAULT_INT)]
    interveal: u64,
    #[structopt(short, long, default_value = DEFAULT_DURATION)]
    duration: u64,
}

#[async_std::main]
async fn main() {
    env_logger::init();

    let args = CallArgs::from_args();

    let count: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));

    let (sender, receiver) = flume::unbounded::<Arc<Vec<u8>>>();
    // println!("layer,scenario,test,name,size,messages");

    let c = count.clone();
    let i = args.interveal.clone();
    let s = args.size.clone();
    task::spawn(async move {
        loop {
            task::sleep(Duration::from_secs(i)).await;
            let n = c.swap(0, Ordering::AcqRel);
            let msgs = n / i;
            println!("flume,same-runtime,throughput,test-name,{},{}", s, msgs);
        }
    });

    task::spawn(async move {
        while let Ok(_) = receiver.recv_async().await {
            count.fetch_add(1, Ordering::AcqRel);
        }
    });

    let d = args.duration;
    task::spawn(async move {
        task::sleep(Duration::from_secs(d)).await;
        std::process::exit(0);
    });

    let data = Arc::new(vec![0; args.size as usize]);

    loop {
        sender.send_async(Arc::clone(&data)).await;
    }
}
