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

use std::net::IpAddr;
use std::num::NonZeroUsize;
use zenoh::prelude::ZInt;
use zenoh_buffers::ZSlice;
use zenoh_protocol::common::Attachment;
use zenoh_protocol::core::Channel;
use zenoh_protocol::transport::{Close, InitAck, InitSyn, Join, KeepAlive, OpenAck, OpenSyn};
use zenoh_protocol::zenoh::{Data, Declare, LinkStateList, Pull, Query, RoutingContext, Unit};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TCPEndpoint {
    pub addr: IpAddr,
    pub port: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ZenohStatistics {
    pub total_data_payload: usize,
    pub total_query_paylaod: usize,
    pub total_data: usize,
    pub total_data_bytes: usize,
    pub total_unit: usize,
    pub total_unit_bytes: usize,
    pub total_pull: usize,
    pub total_pull_bytes: usize,
    pub total_query: usize,
    pub total_query_bytes: usize,
    pub total_declare: usize,
    pub total_declare_bytes: usize,
    pub total_linkstate: usize,
    pub total_linkstate_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TransportStatistics {
    pub total_frames: usize,
    pub total_frames_bytes: usize,
    pub total_initsyn: usize,
    pub total_initsyn_bytes: usize,
    pub total_initack: usize,
    pub total_initack_bytes: usize,
    pub total_opensyn: usize,
    pub total_opensyn_bytes: usize,
    pub total_openack: usize,
    pub total_openack_bytes: usize,
    pub total_join: usize,
    pub total_join_bytes: usize,
    pub total_close: usize,
    pub total_close_bytes: usize,
    pub total_keepalive: usize,
    pub total_keepalive_bytes: usize,
    pub total_tx_msgs: usize,
    pub total_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SessionStatistics {
    pub zenoh: ZenohStatistics,
    pub transport: TransportStatistics,
    pub overhead: usize,
}

// Zenoh Transport and Messages wrappers

pub struct SizedTransportMessage {
    pub body: SizedTransportBody,
    pub attachment: Option<Attachment>,
    pub size: NonZeroUsize,
}

pub struct SizedInitSyn(pub NonZeroUsize, pub InitSyn);
pub struct SizedInitAck(pub NonZeroUsize, pub InitAck);
pub struct SizedOpenSyn(pub NonZeroUsize, pub OpenSyn);
pub struct SizedOpenAck(pub NonZeroUsize, pub OpenAck);
pub struct SizedJoin(pub NonZeroUsize, pub Join);
pub struct SizedClose(pub NonZeroUsize, pub Close);
pub struct SizedKeepAlive(pub NonZeroUsize, pub KeepAlive);

pub enum SizedTransportBody {
    InitSyn(SizedInitSyn),
    InitAck(SizedInitAck),
    OpenSyn(SizedOpenSyn),
    OpenAck(SizedOpenAck),
    Join(SizedJoin),
    Close(SizedClose),
    KeepAlive(SizedKeepAlive),
    Frame(SizedFrame),
}

pub struct SizedFrame {
    pub channel: Channel,
    pub sn: ZInt,
    pub payload: SizedFramePayload,
    pub size: NonZeroUsize,
}

pub enum SizedFramePayload {
    Fragment { buffer: ZSlice, is_final: bool },
    Messages { messages: Vec<SizedZenohMessage> },
}

pub struct SizedZenohMessage {
    pub body: SizedZenohBody,
    pub channel: Channel,
    pub routing_context: Option<RoutingContext>,
    pub attachment: Option<Attachment>,
}

pub struct SizedData(pub NonZeroUsize, pub Data);
pub struct SizedUnit(pub NonZeroUsize, pub Unit);
pub struct SizedPull(pub NonZeroUsize, pub Pull);
pub struct SizedQuery(pub NonZeroUsize, pub Query);
pub struct SizedDeclare(pub NonZeroUsize, pub Declare);
pub struct SizedLinkStateList(pub NonZeroUsize, pub LinkStateList);

pub enum SizedZenohBody {
    Data(SizedData),
    Unit(SizedUnit),
    Pull(SizedPull),
    Query(SizedQuery),
    Declare(SizedDeclare),
    LinkStateList(SizedLinkStateList),
}
