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

const ZENOH_UNICAST_PORT: u16 = 7447;
const ZENOH_MULTICAST_PORT: u16 = 7446;
const ZENOH_FRAME_SIZE_LEN: usize = 2;
const IP_HDR_BLOCK_LEN: usize = 4;
const TCP_HDR_LEN: usize = 20;

pub mod analysis;
pub mod types;
pub mod utils;
