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

use async_std::stream::StreamExt;
use clap::Parser;
use rand::Rng;
use std::io::{self, Write};
use std::time::Duration;
use zenoh::net::protocol::io::SplitBuffer;
use zenoh::prelude::*;
use zenoh::publication::CongestionControl;
use zenoh_flow::{Data, Message};
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
    ping: bool,
    #[clap(short, long)]
    udp: bool,
}

async fn ping(interval: f64, session: zenoh::Session) {
    let key_expr_ping = session.declare_expr("/test/latency/ping").await.unwrap();
    let key_expr_pong = session.declare_expr("/test/latency/pong").await.unwrap();

    let sub = session.subscribe(&key_expr_pong).await.unwrap();

    loop {
        async_std::task::sleep(Duration::from_secs_f64(interval)).await;
        let hlc = async_std::sync::Arc::new(uhlc::HLC::default());

        let msg = Latency { ts: get_epoch_us() };
        let data = Data::from::<Latency>(msg);
        let msg = Message::from_serdedata(data, hlc.new_timestamp(), vec![], vec![]);

        let value = msg.serialize_bincode().unwrap();

        session
            .put(&key_expr_ping, value)
            .congestion_control(CongestionControl::Block)
            .await
            .unwrap();

        let _ = sub.recv();
    }
}

async fn pong(session: zenoh::Session, msgs: u64, pipeline: u64, udp: bool) {
    let key_expr_ping = session.declare_expr("/test/latency/ping").await.unwrap();
    let key_expr_pong = session.declare_expr("/test/latency/pong").await.unwrap();
    let pong_data: Vec<u8> = vec![];
    let mut sub = session.subscribe(&key_expr_ping).await.unwrap();
    let layer = match udp {
        true => "zenoh-udp",
        false => "zenoh",
    };

    while let Some(msg) = sub.receiver().next().await {
        let now = get_epoch_us();
        let de: Message = bincode::deserialize(&msg.value.payload.contiguous()).unwrap();

        match de {
            Message::Data(mut data_msg) => {
                let data = data_msg.get_inner_data().try_get::<Latency>().unwrap();
                let elapsed = now - data.ts;

                // framework,scenario,test,pipeline,payload,rate,value,unit
                println!("{layer},multi,latency,{pipeline},8,{msgs},{elapsed},us");

                io::stdout().flush().unwrap();

                session
                    .put(&key_expr_pong, pong_data.clone())
                    .congestion_control(CongestionControl::Block)
                    .await
                    .unwrap();
            }
            _ => (),
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
    } else {
        let locator = format!("tcp/127.0.0.1:{}", rng.gen_range(8000..65000));
        config
            .listen
            .set_endpoints(vec![locator.parse().unwrap()])
            .unwrap();
    }

    let session = zenoh::open(config).await.unwrap();

    if args.ping {
        ping(interval, session).await;
    } else {
        pong(session, args.msgs, args.length, args.udp).await;
    }
}
