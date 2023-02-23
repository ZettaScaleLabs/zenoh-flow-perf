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

use std::collections::HashMap;

use crate::types::{
    SessionStatistics, SizedFrame, SizedFramePayload, SizedTransportBody, SizedTransportMessage,
    SizedZenohBody, SizedZenohMessage, TCPEndpoint, TransportStatistics, ZenohStatistics,
};
use crate::utils::read_messages;
use zenoh_buffers::reader::HasReader;
use zenoh_buffers::reader::Reader;
use zenoh_buffers::SplitBuffer;
use zenoh_codec::RCodec;
use zenoh_codec::Zenoh060;

pub fn analyze_zenoh_tcp(
    tcp_sessions: HashMap<(TCPEndpoint, TCPEndpoint), Vec<u8>>,
) -> HashMap<(TCPEndpoint, TCPEndpoint), SessionStatistics> {
    let mut zenoh_sessions: HashMap<(TCPEndpoint, TCPEndpoint), SessionStatistics> = HashMap::new();
    if tcp_sessions.len() > 0 {
        for ((src, dst), data) in tcp_sessions {
            log::debug!("[Session] - Analyzing - From: {src:?} To: {dst:?}");

            let mut total_payload = 0;
            let mut total_query_payload = 0;

            // Zenoh Messages
            let mut total_data = 0;
            let mut total_unit = 0;
            let mut total_pull = 0;
            let mut total_query = 0;
            let mut total_declare = 0;
            let mut total_linkstate = 0;

            let mut total_data_bytes = 0;
            let mut total_unit_bytes = 0;
            let mut total_pull_bytes = 0;
            let mut total_query_bytes = 0;
            let mut total_declare_bytes = 0;
            let mut total_linkstate_bytes = 0;

            // Transport Messages
            let mut total_frames = 0;
            let mut total_initsyn = 0;
            let mut total_initack = 0;
            let mut total_opensyn = 0;
            let mut total_openack = 0;
            let mut total_join = 0;
            let mut total_close = 0;
            let mut total_keepalive = 0;
            let mut total_tx_msgs = 0;

            let mut total_frames_bytes = 0;
            let mut total_initsyn_bytes = 0;
            let mut total_initack_bytes = 0;
            let mut total_opensyn_bytes = 0;
            let mut total_openack_bytes = 0;
            let mut total_join_bytes = 0;
            let mut total_close_bytes = 0;
            let mut total_keepalive_bytes = 0;

            let mut total_bytes = 0;

            let mut deserialized: Vec<SizedTransportMessage> = vec![];
            let mut zmsgs: Vec<SizedZenohMessage> = vec![];

            let codec = Zenoh060::default();

            let slices = read_messages(data);

            for mut slice in slices {
                let mut reader = slice.reader();
                while reader.can_read() {
                    match codec.read(&mut reader) {
                        Ok(msg) => deserialized.push(msg),
                        Err(_e) => (),
                    }
                }
            }

            for msg in deserialized.drain(..) {
                total_bytes += msg.size.get();
                total_tx_msgs += 1;
                // log::debug!("[Parsing] Zenoh Frame message is {msg:?}");
                match msg.body {
                    SizedTransportBody::Frame(SizedFrame { payload, size, .. }) => {
                        total_frames += 1;
                        total_frames_bytes += size.get();
                        match payload {
                            SizedFramePayload::Messages { mut messages } => {
                                zmsgs.append(&mut messages)
                            }
                            _ => (),
                        }
                    }
                    SizedTransportBody::InitSyn(tb) => {
                        total_initsyn += 1;
                        total_initsyn_bytes += tb.0.get();
                    }
                    SizedTransportBody::InitAck(tb) => {
                        total_initack += 1;
                        total_initack_bytes += tb.0.get();
                    }
                    SizedTransportBody::OpenSyn(tb) => {
                        total_opensyn += 1;
                        total_opensyn_bytes += tb.0.get();
                    }
                    SizedTransportBody::OpenAck(tb) => {
                        total_openack += 1;
                        total_openack_bytes += tb.0.get();
                    }
                    SizedTransportBody::Join(tb) => {
                        total_join += 1;
                        total_join_bytes += tb.0.get();
                    }
                    SizedTransportBody::Close(tb) => {
                        total_close += 1;
                        total_close_bytes += tb.0.get();
                    }
                    SizedTransportBody::KeepAlive(tb) => {
                        total_keepalive += 1;
                        total_keepalive_bytes += tb.0.get();
                    }
                }
            }

            for msg in zmsgs {
                match msg.body {
                    SizedZenohBody::Data(data_msg) => {
                        total_data += 1;
                        total_payload += data_msg.1.payload.len();
                        total_data_bytes += data_msg.0.get();
                    }
                    SizedZenohBody::Unit(unit_msg) => {
                        total_unit += 1;
                        total_unit_bytes += unit_msg.0.get();
                    }
                    SizedZenohBody::Pull(pull_msg) => {
                        total_pull += 1;
                        total_pull_bytes += pull_msg.0.get();
                    }
                    SizedZenohBody::Query(query_msg) => {
                        total_query += 1;
                        total_query_bytes += query_msg.0.get();
                        match query_msg.1.body {
                            Some(body) => total_query_payload += body.payload.len(),
                            _ => (),
                        }
                    }
                    SizedZenohBody::Declare(declare_msg) => {
                        total_declare += 1;
                        total_declare_bytes += declare_msg.0.get();
                    }
                    SizedZenohBody::LinkStateList(linkstate_msg) => {
                        total_linkstate += 1;
                        total_linkstate_bytes += linkstate_msg.0.get();
                    }
                }
            }

            let overhead = total_bytes - total_payload - total_query_payload;

            log::debug!("[Session] - From: {src:?} To: {dst:?} Total bytes: {total_bytes} Total Payload: {total_payload} Overhead: {overhead}");

            let zenoh_statistics = ZenohStatistics {
                total_data_payload: total_payload,
                total_query_paylaod: total_query_payload,
                total_data: total_data,
                total_data_bytes,
                total_unit: total_unit,
                total_unit_bytes,
                total_pull: total_pull,
                total_pull_bytes,
                total_query: total_query,
                total_query_bytes,
                total_declare: total_declare,
                total_declare_bytes,
                total_linkstate: total_linkstate,
                total_linkstate_bytes,
            };

            let transport_statistics = TransportStatistics {
                total_frames,
                total_frames_bytes,
                total_initsyn,
                total_initsyn_bytes,
                total_initack,
                total_initack_bytes,
                total_opensyn,
                total_opensyn_bytes,
                total_openack,
                total_openack_bytes,
                total_join,
                total_join_bytes,
                total_close,
                total_close_bytes,
                total_keepalive,
                total_keepalive_bytes,
                total_tx_msgs,
                total_bytes,
            };

            let session_statistics = SessionStatistics {
                zenoh: zenoh_statistics,
                transport: transport_statistics,
                overhead,
            };

            zenoh_sessions.insert((src, dst), session_statistics);
        }
    }

    zenoh_sessions
}
