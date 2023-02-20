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
    SessionStatistics, SizedTransportMessage, TCPEndpoint, TransportStatistics, ZenohStatistics,
};
use crate::utils::read_messages;
use zenoh_buffers::reader::HasReader;
use zenoh_buffers::reader::Reader;
use zenoh_buffers::SplitBuffer;
use zenoh_codec::RCodec;
use zenoh_codec::Zenoh060;
use zenoh_protocol::transport::{Frame, FramePayload, TransportBody};
use zenoh_protocol::zenoh::{ZenohBody, ZenohMessage};

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

            let mut total_bytes = 0;

            let mut deserialized: Vec<SizedTransportMessage> = vec![];
            let mut zmsgs: Vec<ZenohMessage> = vec![];

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

            for smsg in deserialized.drain(..) {
                let SizedTransportMessage(msg, msg_size) = smsg;
                total_bytes += msg_size.get();
                total_tx_msgs += 1;
                log::debug!("[Parsing] Zenoh Frame message is {msg:?}");
                match msg.body {
                    TransportBody::Frame(Frame { payload, .. }) => {
                        total_frames += 1;
                        match payload {
                            FramePayload::Messages { mut messages } => zmsgs.append(&mut messages),
                            _ => (),
                        }
                    }
                    TransportBody::InitSyn(_) => total_initsyn += 1,
                    TransportBody::InitAck(_) => total_initack += 1,
                    TransportBody::OpenSyn(_) => total_opensyn += 1,
                    TransportBody::OpenAck(_) => total_openack += 1,
                    TransportBody::Join(_) => total_join += 1,
                    TransportBody::Close(_) => total_close += 1,
                    TransportBody::KeepAlive(_) => total_keepalive += 1,
                }
            }

            for msg in zmsgs {
                match msg.body {
                    ZenohBody::Data(data_msg) => {
                        total_data += 1;
                        total_payload += data_msg.payload.len();
                    }
                    ZenohBody::Unit(_unit_msg) => {
                        total_unit += 1;
                    }
                    ZenohBody::Pull(_pull_msg) => total_pull += 1,
                    ZenohBody::Query(query_msg) => {
                        total_query += 1;
                        match query_msg.body {
                            Some(body) => total_query_payload += body.payload.len(),
                            _ => (),
                        }
                    }
                    ZenohBody::Declare(_declare_msg) => total_declare += 1,
                    ZenohBody::LinkStateList(_linkstate_msg) => total_linkstate += 1,
                }
            }

            let overhead = total_bytes - total_payload - total_query_payload;

            log::debug!("[Session] - From: {src:?} To: {dst:?} Total bytes: {total_bytes} Total Payload: {total_payload} Overhead: {overhead}");

            let zenoh_statistics = ZenohStatistics {
                total_data_payload: total_payload,
                total_query_paylaod: total_query_payload,
                total_data: total_data,
                total_data_bytes: 0,
                total_unit: total_unit,
                total_unit_bytes: 0,
                total_pull: total_pull,
                total_pull_bytes: 0,
                total_query: total_query,
                total_query_bytes: 0,
                total_declare: total_declare,
                total_declare_bytes: 0,
                total_linkstate: total_linkstate,
                total_linkstate_bytes: 0,
            };

            let transport_statistics = TransportStatistics {
                total_frames,
                total_frames_bytes: 0,
                total_initsyn,
                total_initsyn_bytes: 0,
                total_initack,
                total_initack_bytes: 0,
                total_opensyn,
                total_opensyn_bytes: 0,
                total_openack,
                total_openack_bytes: 0,
                total_join,
                total_join_bytes: 0,
                total_close,
                total_close_bytes: 0,
                total_keepalive,
                total_keepalive_bytes: 0,
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
