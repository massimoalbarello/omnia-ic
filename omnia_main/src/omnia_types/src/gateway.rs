use std::collections::BTreeMap;
use candid::{CandidType, Deserialize};
use serde::Serialize;


use crate::{environment::EnvironmentUID, errors::GenericError};
use crate::device::{DeviceUID, StoredDeviceInfo};

pub type GatewayUID = String;
pub type GatewayPrincipald = String;
pub type GatewayIp = String;
pub type GatewayPrincipalId = String;

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct StoredRegisteredGateway {
    pub gateway_name: String,
    pub devices: BTreeMap<DeviceUID, StoredDeviceInfo>,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct GatewayRegistrationInput {
    pub env_uid: EnvironmentUID,
    pub gateway_name: String,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct RegisteredGateway {
    pub gateway_name: String,
    pub gateway_ip: GatewayIp,
    pub env_uid: EnvironmentUID,

}

pub type RegisteredGatewayResult = Result<RegisteredGateway, GenericError>;
pub type MultipleRegisteredGatewayResult = Result<Vec<RegisteredGateway>, GenericError>;
