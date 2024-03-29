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
use rand::Rng;
use std::time::Duration;
use zenoh::prelude::r#async::*;
use zenoh::publication::CongestionControl;
use zenoh_flow::prelude::*;
use zenoh_flow_perf::{get_epoch_us, Latency};

static DEFAULT_PIPELINE: &str = "1";
static DEFAULT_MSGS: &str = "1";

#[derive(Parser, Debug)]
struct CallArgs {
    /// Config file
    #[clap(short, long, default_value = DEFAULT_PIPELINE)]
    length: u64,
    #[clap(short, long, default_value = DEFAULT_MSGS)]
    msgs: u64,
    #[clap(short, long)]
    publisher: bool,
    #[clap(short, long)]
    udp: bool,
    #[clap(short, long)]
    drop: bool,
}

async fn publisher(interval: f64, session: zenoh::Session, drop: bool) {
    let reskey = String::from("test/latency");

    loop {
        async_std::task::sleep(Duration::from_secs_f64(interval)).await;
        let hlc = async_std::sync::Arc::new(uhlc::HLC::default());

        let msg = Latency { ts: get_epoch_us() };
        let data = Data::from(msg);
        let msg = Message::from_serdedata(data, hlc.new_timestamp());

        let value = msg.serialize_bincode().unwrap();
        if drop {
            session
                .put(&reskey, value)
                .congestion_control(CongestionControl::Drop)
                .res()
                .await
                .unwrap();
        } else {
            session
                .put(&reskey, value)
                .congestion_control(CongestionControl::Block)
                .res()
                .await
                .unwrap();
        }
    }
}

async fn subscriber(session: zenoh::Session, msgs: u64, pipeline: u64, udp: bool) {
    let reskey = String::from("test/latency");
    let sub = session.declare_subscriber(&reskey).res().await.unwrap();
    let layer = match udp {
        true => "zenoh-lat-udp",
        false => "zenoh-lat",
    };

    while let Ok(msg) = sub.recv_async().await {
        let now = get_epoch_us();
        let de: Message = bincode::deserialize(&msg.value.payload.contiguous()).unwrap();

        if let Message::Data(mut data_msg) = de {
            let data = data_msg.get_inner_data().try_get::<Latency>().unwrap();
            let elapsed = now - data.ts;

            // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
            println!("{layer},scenario,latency,pipeline,{msgs},{pipeline},{elapsed},us");
        }
    }
}

#[async_std::main]
async fn main() {
    env_logger::init();

    let mut rng = rand::thread_rng();

    let args = CallArgs::parse();

    let interval = 1.0 / (args.msgs as f64);

    let mut config = zenoh::config::Config::default();
    config
        .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
        .unwrap();
    if args.udp {
        let locator = format!("udp/127.0.0.1:{}", rng.gen_range(8000..65000));
        config
            .listen
            .set_endpoints(vec![locator.parse().unwrap()])
            .unwrap();
    }

    let session = zenoh::open(config).res().await.unwrap();

    if args.publisher {
        publisher(interval, session, args.drop).await;
    } else {
        subscriber(session, args.msgs, args.length, args.udp).await;
    }
}
