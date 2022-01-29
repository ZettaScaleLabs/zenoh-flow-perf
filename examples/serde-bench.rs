// //
// // Copyright (c) 2017, 2021 ADLINK Technology Inc.
// //
// // This program and the accompanying materials are made available under the
// // terms of the Eclipse Public License 2.0 which is available at
// // http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// // which is available at https://www.apache.org/licenses/LICENSE-2.0.
// //
// // SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
// //
// // Contributors:
// //   ADLINK zenoh team, <zenoh@adlink-labs.tech>
// //

// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use std::thread;
// use std::time::{Duration, Instant};
// use structopt::StructOpt;
// use zenoh_flow::{Data, Message};

// static DEFAULT_SIZE: &str = "1";
// static DEFAULT_SAMPLES: &str = "10000000";

// #[derive(StructOpt, Debug)]
// struct CallArgs {
//     #[structopt(short, long, default_value = DEFAULT_SIZE)]
//     size: u64,
//     #[structopt(short, long)]
//     de: bool,
//     #[structopt(short, long, default_value = DEFAULT_SAMPLES)]
//     samples: u64,
// }

// fn bench_serialize(message: &Message) {
//     let _ = message.serialize_bincode().unwrap();
// }

// fn bench_deserialize(data: &[u8]) {
//     let _: Message = bincode::deserialize(data).unwrap();
// }

// fn ser_bench(args: CallArgs) {
//     let flag = Arc::new(AtomicBool::new(false));
//     let hlc = async_std::sync::Arc::new(uhlc::HLC::default());

//     let payload_data = (0u64..args.size)
//         .map(|i| (i % 10) as u8)
//         .collect::<Vec<u8>>();

//     let data = Data::from_bytes(payload_data);
//     let zf_msg = Message::from_serdedata(data, hlc.new_timestamp(), vec![], vec![]);
//     let mut samples: Vec<u128> = Vec::with_capacity(args.samples as usize);

//     // let f = flag.clone();
//     // let _handler = thread::spawn( move || {
//     //     thread::sleep(Duration::from_secs(5));
//     //     f.swap(true, Ordering::Relaxed);
//     // });

//     //warm up 5s
//     // while !flag.load(Ordering::Relaxed){
//     for _ in 0..args.samples * 5 {
//         bench_serialize(&zf_msg);
//     }

//     // layer,scenario name,test kind, test name, payload size, latency
//     for _ in 0..args.samples {
//         let now = Instant::now();
//         bench_serialize(&zf_msg);
//         let duration = now.elapsed().as_nanos();
//         samples.push(duration);
//     }

//     // print at the end
//     for sample in samples {
//         println!(
//             "ser,serde,serialization,serialization,{},{}",
//             args.size, sample
//         );
//     }
// }

// fn de_bench(args: CallArgs) {
//     let hlc = async_std::sync::Arc::new(uhlc::HLC::default());

//     let payload_data = (0u64..args.size)
//         .map(|i| (i % 10) as u8)
//         .collect::<Vec<u8>>();

//     let data = Data::from_bytes(payload_data);
//     let zf_msg = Message::from_serdedata(data, hlc.new_timestamp(), vec![], vec![]);

//     let serialized_zf = zf_msg.serialize_bincode().unwrap();
//     let mut samples: Vec<u128> = Vec::with_capacity(args.samples as usize);
//     //warm up 5s

//     // layer,scenario name,test kind, test name, payload size, latency
//     for _ in 0..args.samples {
//         let now = Instant::now();
//         bench_deserialize(&serialized_zf);
//         let duration = now.elapsed().as_micros();
//         samples.push(duration);

//         //println!("de,serde,deserialization,deserialization,{},{}", args.size, duration);
//     }

//     // print at the end
//     for sample in samples {
//         println!(
//             "de,serde,deserialization,deserialization,{},{}",
//             args.size, sample
//         );
//     }
// }

// fn main() {
//     let args = CallArgs::from_args();

//     if !args.de {
//         ser_bench(args)
//     } else {
//         de_bench(args)
//     }
// }
fn main() {
    println!("Nope")
}
