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

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use zenoh_flow::prelude::*;
use zenoh_flow::zenoh_flow_derive::ZFData;

pub mod nodes;
pub mod runtime;

pub fn write_string_to_file(content: String, filename: &str) {
    let path = Path::new(filename);
    let mut write_file = File::create(path).unwrap();
    write!(write_file, "{content}").unwrap();
}

#[derive(Debug, Clone, ZFData, Serialize, Deserialize)]
pub struct ThrData {
    pub data: Vec<u8>,
}

impl ZFData for ThrData {
    fn try_serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)
            .map_err(|e| zferror!(ErrorKind::SerializationError, "{}", e))?
            .to_vec())
    }
    fn try_deserialize(bytes: &[u8]) -> Result<ThrData>
    where
        Self: Sized,
    {
        bincode::deserialize::<ThrData>(bytes)
            .map_err(|e| zferror!(ErrorKind::DeserializationError, "{}", e).into())
    }
}

#[derive(Debug, Clone, ZFData, Serialize, Deserialize)]
pub struct LatData {
    pub data: Vec<u8>,
    pub ts: u128,
}

impl ZFData for LatData {
    fn try_serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)
            .map_err(|e| zferror!(ErrorKind::SerializationError, "{}", e))?
            .to_vec())
    }

    fn try_deserialize(bytes: &[u8]) -> Result<LatData>
    where
        Self: Sized,
    {
        bincode::deserialize::<LatData>(bytes)
            .map_err(|e| zferror!(ErrorKind::DeserializationError, "{}", e).into())
    }
}

pub fn get_epoch_us() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}





#[derive(Debug, Clone, ZFData, Serialize, Deserialize)]
pub struct Latency {
    pub ts: u128,
}

impl ZFData for Latency {
    fn try_serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)
            .map_err(|e| zferror!(ErrorKind::SerializationError, "{}", e))?
            .to_vec())
    }

    fn try_deserialize(bytes: &[u8]) -> Result<Latency>
    where
        Self: Sized,
    {
        bincode::deserialize::<Latency>(bytes)
            .map_err(|e| zferror!(ErrorKind::DeserializationError, "{}", e).into())
    }
}

#[derive(Debug, Clone, ZFData, Serialize, Deserialize)]
pub struct CriterionData {
    pub d: u64,
}

impl ZFData for CriterionData {
    fn try_serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)
            .map_err(|e| zferror!(ErrorKind::SerializationError, "{}", e))?
            .to_vec())
    }

    fn try_deserialize(bytes: &[u8]) -> Result<CriterionData>
    where
        Self: Sized,
    {
        bincode::deserialize::<CriterionData>(bytes)
            .map_err(|e| zferror!(ErrorKind::DeserializationError, "{}", e).into())
    }
}
