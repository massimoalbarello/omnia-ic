use std::collections::BTreeMap;
use candid::{CandidType, Deserialize};

use super::interface_types::{self as InterfaceTypes};

type PrincipalId = String;
type EnvironmentUID = u32;
type GatewayUID = u32;
type DeviceUID = u32;

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct DeviceInfo {
    pub device_name: String,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct GatewayInfo {
    pub gateway_name: String,
    pub gateway_uid: GatewayUID,
    pub devices: BTreeMap<DeviceUID, DeviceInfo>,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct EnvironmentInfo {
    pub env_name: String,
    pub env_uid: EnvironmentUID,
    pub env_gateways: BTreeMap<GatewayUID, GatewayInfo>,
    pub env_manager_principal_id: PrincipalId,
}

impl EnvironmentInfo {
    pub fn update(
        &mut self,
        entity: InterfaceTypes::EntityRegistrationInput,
        uid: u32
    ) -> InterfaceTypes::EntityRegistrationResult {
        match entity {
            InterfaceTypes::EntityRegistrationInput::Gateway(gateway) => {
                ic_cdk::print(format!("Created gateway with UID: {:?}", uid));

                self.env_gateways.insert(
                    uid,
                    GatewayInfo {
                        gateway_name: gateway.gateway_name.clone(),
                        gateway_uid: uid,
                        devices: BTreeMap::new(),
                    }
                );

                InterfaceTypes::EntityRegistrationResult::Gateway(
                    InterfaceTypes::GatewayRegistrationResult {
                        gateway_name: gateway.gateway_name,
                        gateway_uid: uid,
                    }
                )
            },
            InterfaceTypes::EntityRegistrationInput::Device(device) => {
                match self.env_gateways.remove(&device.gateway_uid) {
                    Some(mut gateway_info) => {
                    ic_cdk::print(format!("Created device with UID: {:?}", uid));

                        gateway_info.devices.insert(
                            uid,
                            DeviceInfo {
                                device_name: device.device_name.clone(),
                            }
                        );
    
                        self.env_gateways.insert(
                            device.gateway_uid,
                            gateway_info
                        );

                        InterfaceTypes::EntityRegistrationResult::Device(
                            InterfaceTypes::DeviceRegistrationResult {
                                device_name: device.device_name,
                                device_uid: uid,
                            }
                        )
                    },
                    None => panic!("Gateway does not exist in environment"),
                }
            },
        }
    }
}