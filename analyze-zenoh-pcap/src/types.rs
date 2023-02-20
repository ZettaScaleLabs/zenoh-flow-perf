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
use zenoh_protocol::transport::TransportMessage;

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

pub struct SizedTransportMessage(pub TransportMessage, pub NonZeroUsize);
