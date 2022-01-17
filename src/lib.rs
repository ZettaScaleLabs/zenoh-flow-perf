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

use std::time::{SystemTime, UNIX_EPOCH};
use zenoh_flow::serde::{Deserialize, Serialize};
use zenoh_flow::zenoh_flow_derive::ZFData;
use zenoh_flow::{Deserializable, ZFData, ZFError, ZFResult};

#[derive(Debug, Clone, ZFData, Serialize, Deserialize)]
pub struct ThrData {
    pub data: Vec<u8>,
}

impl ZFData for ThrData {
    fn try_serialize(&self) -> zenoh_flow::ZFResult<Vec<u8>> {
        Ok(bincode::serialize(self)
            .map_err(|_| ZFError::SerializationError)?
            .to_vec())
    }
}

impl Deserializable for ThrData {
    fn try_deserialize(bytes: &[u8]) -> ZFResult<ThrData>
    where
        Self: Sized,
    {
        bincode::deserialize::<ThrData>(bytes).map_err(|_| ZFError::DeseralizationError)
    }
}

#[derive(Debug, Clone, ZFData, Serialize, Deserialize)]
pub struct LatData {
    pub data: Vec<u8>,
    pub ts: u128,
}

impl ZFData for LatData {
    fn try_serialize(&self) -> zenoh_flow::ZFResult<Vec<u8>> {
        Ok(bincode::serialize(self)
            .map_err(|_| ZFError::SerializationError)?
            .to_vec())
    }
}

impl Deserializable for LatData {
    fn try_deserialize(bytes: &[u8]) -> ZFResult<LatData>
    where
        Self: Sized,
    {
        bincode::deserialize::<LatData>(bytes).map_err(|_| ZFError::DeseralizationError)
    }
}

pub fn get_epoch_us() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

#[derive(Debug, Clone, ZFData, Serialize, Deserialize)]
pub struct Latency {
    pub ts: u128,
}

impl ZFData for Latency {
    fn try_serialize(&self) -> zenoh_flow::ZFResult<Vec<u8>> {
        Ok(bincode::serialize(self)
            .map_err(|_| ZFError::SerializationError)?
            .to_vec())
    }
}

impl Deserializable for Latency {
    fn try_deserialize(bytes: &[u8]) -> ZFResult<Latency>
    where
        Self: Sized,
    {
        bincode::deserialize::<Latency>(bytes).map_err(|_| ZFError::DeseralizationError)
    }
}
