use candid::{CandidType, Deserialize};

type EnvironmentUID = u32;
type GatewayUID = u32;
type DeviceUID = u32;



#[derive(Debug, CandidType, Deserialize)]
pub struct EnvironmentCreationInput {
    pub env_name: String,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct EnvironmentCreationResult {
    pub env_name: String,
    pub env_uid: EnvironmentUID,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct GatewayRegistrationInput {
    pub env_uid: EnvironmentUID,
    pub gateway_name: String,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct GatewayRegistrationResult {
    pub gateway_name: String,
    pub gateway_uid: GatewayUID,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct DeviceRegistrationInput {
    pub env_uid: EnvironmentUID,
    pub gateway_uid: GatewayUID,
    pub device_name: String,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct DeviceRegistrationResult {
    pub device_name: String,
    pub device_uid: DeviceUID,
}

#[derive(CandidType, Deserialize)]
pub enum EntityRegistrationInput {
    Gateway(GatewayRegistrationInput),
    Device(DeviceRegistrationInput),
}

impl EntityRegistrationInput {
    pub fn getEnvironmentUID(&self) -> EnvironmentUID {
        match self {
            EntityRegistrationInput::Gateway(gateway) => gateway.env_uid,
            EntityRegistrationInput::Device(device) => device.env_uid,
        }
    }
}

#[derive(Debug, CandidType, Deserialize)]
pub enum EntityRegistrationResult {
    Gateway(GatewayRegistrationResult),
    Device(DeviceRegistrationResult),
} 