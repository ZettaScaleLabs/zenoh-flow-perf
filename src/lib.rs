use zenoh_flow::serde::{Deserialize, Serialize};
use zenoh_flow::zenoh_flow_derive::{ZFData};
use zenoh_flow::{ZFDataTrait, ZFDeserializable, ZFError, ZFResult};

#[derive(Debug, Clone, ZFData, Serialize, Deserialize)]
pub struct ThrData{
    pub data : Vec<u8>
}

impl ZFDataTrait for ThrData {
    fn try_serialize(&self) -> zenoh_flow::ZFResult<Vec<u8>> {
        Ok(bincode::serialize(self).map_err(|_| ZFError::SerializationError)?.to_vec())
    }
}

impl ZFDeserializable for ThrData {
    fn try_deserialize(bytes: &[u8]) -> ZFResult<ThrData>
    where
        Self: Sized,
    {
        Ok(
            bincode::deserialize::<ThrData>(bytes).map_err(|_| ZFError::DeseralizationError)?,
        )
    }
}