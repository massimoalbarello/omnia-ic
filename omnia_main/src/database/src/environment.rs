use std::collections::BTreeMap;
use candid::candid_method;
use ic_cdk::print;
use ic_cdk_macros::update;
use omnia_types::{
    environment::{EnvironmentCreationInput, EnvironmentCreationResult, EnvironmentIndex, EnvironmentValue, EnvironmentUidIndex, EnvironmentUidValue, EnvironmentUID},
    gateway::{
        RegisteredGatewayResult, GatewayRegistrationInput,
        MultipleRegisteredGatewayResult, GatewayPrincipalId, InitializedGatewayIndex, InitializedGatewayValue, RegisteredGatewayIndex, RegisteredGatewayValue,
    },
    virtual_persona::{VirtualPersonaPrincipalId, VirtualPersonaIndex}, http::{IpChallengeNonce, IpChallengeIndex},
    errors::GenericResult
};

use crate::{uuid::generate_uuid, STATE};

#[update(name = "initGatewayByIp")]
#[candid_method(update, rename = "initGatewayByIp")]
async fn init_gateway_by_ip(nonce: IpChallengeNonce, gateway_principal_id: GatewayPrincipalId) -> GenericResult<GatewayPrincipalId> {

    let ip_challenge_index = IpChallengeIndex {
        nonce,
    };

    STATE.with(|state| {   
        let ip_challenge_value = state.borrow_mut().ip_challenges.validate_ip_challenge(&ip_challenge_index)?;

        let initialized_gateway_index = InitializedGatewayIndex {
            ip: ip_challenge_value.requester_ip,
        };

        let initialized_gateway_value = InitializedGatewayValue {
            principal_id: gateway_principal_id.clone()
        };

        state.borrow_mut().initialized_gateways.create(initialized_gateway_index, initialized_gateway_value).expect("previous entry should not exist");
        Ok(gateway_principal_id)
    })
}

#[update(name = "getInitializedGatewaysByIp")]
#[candid_method(update, rename = "getInitializedGatewaysByIp")]
async fn get_initialized_gateways_by_ip(nonce: IpChallengeNonce) -> GenericResult<Vec<InitializedGatewayValue>> {

    let ip_challenge_index = IpChallengeIndex {
        nonce,
    };

    STATE.with(|state| {
        let ip_challenge_value = state.borrow_mut().ip_challenges.validate_ip_challenge(&ip_challenge_index)?;

        let initialized_gateway_index = InitializedGatewayIndex {
            ip: ip_challenge_value.requester_ip,
        };

        match state.borrow_mut().initialized_gateways.read(&initialized_gateway_index) {
            Ok(initialized_gateway_value) => Ok(vec![initialized_gateway_value.to_owned()]),
            Err(e) => Err(e),

        }
    })
}

#[update(name = "createNewEnvironment")]
#[candid_method(update, rename = "createNewEnvironment")]
async fn create_new_environment(
    environment_manager_principal_id: VirtualPersonaPrincipalId,
    environment_creation_input: EnvironmentCreationInput,
) -> EnvironmentCreationResult {
    print(format!(
        "Creating new environment: {:?} managed by: {:?}",
        environment_creation_input, environment_manager_principal_id
    ));

    let environment_uid = generate_uuid().await;
    print(format!("New environment UID: {:?}", environment_uid));

    let environment_index = EnvironmentIndex {
        environment_uid: environment_uid.clone(),
    };

    let environment_value = EnvironmentValue {
        env_name: environment_creation_input.env_name.clone(),
        env_ip: None,
        env_users_principals_ids: BTreeMap::default(),
        env_gateways_principals_ids: BTreeMap::default(),
        env_manager_principal_id: environment_manager_principal_id,
    };

    STATE.with(|state| {
        state.borrow_mut().environments.create(
            environment_index,
            environment_value
        )
    }).expect("previous entry should not exist");

    let environment_creation_result = EnvironmentCreationResult {
        env_name: environment_creation_input.env_name,
        env_uid: environment_uid,
    };

    print(format!(
        "Created new environment: {:?}",
        environment_creation_result
    ));

    environment_creation_result
}

#[update(name = "registerGatewayInEnvironment")]
#[candid_method(update, rename = "registerGatewayInEnvironment")]
fn register_gateway_in_environment(
    nonce: IpChallengeNonce,
    environment_manager_principal_id: VirtualPersonaPrincipalId,
    gateway_registration_input: GatewayRegistrationInput,
) -> RegisteredGatewayResult {
    
    STATE.with(|state| {

        let ip_challenge_index = IpChallengeIndex {
            nonce,
        };

        let ip_challenge_value = state.borrow_mut().ip_challenges.validate_ip_challenge(&ip_challenge_index)?;

        let initialized_gateway_index = InitializedGatewayIndex {
            ip: ip_challenge_value.requester_ip.clone(),
        };

        let initialized_gateway_value = state.borrow_mut().initialized_gateways.delete(&initialized_gateway_index)?;

        // register mapping IP to Environment UID in order to be able to retrive the UID of the environment from the IP when a User registers in an environment
        let environment_uid_index = EnvironmentUidIndex {
            ip: ip_challenge_value.requester_ip.clone(),
        };

        let environment_uid_value = EnvironmentUidValue {
            env_uid: gateway_registration_input.env_uid.clone(),
        };

        state.borrow_mut().environment_uids.create(environment_uid_index, environment_uid_value).expect("previous entry should not exist");

        print(format!(
            "Registering gateway in environment with UID: {:?} managed by: {:?}",
            gateway_registration_input.env_uid,
            environment_manager_principal_id
        ));

        let registered_gateway_index = RegisteredGatewayIndex {
            principal_id: initialized_gateway_value.principal_id,
        };

        let registered_gateway_value = RegisteredGatewayValue {
            gateway_name: gateway_registration_input.gateway_name,
            gateway_ip: ip_challenge_value.requester_ip,
            env_uid: gateway_registration_input.env_uid.clone(),
        };

        state.borrow_mut().registered_gateways.create(registered_gateway_index.clone(), registered_gateway_value.clone())?;
        

        // add principal ID of registered Gateway to Environment
        let environment_index = EnvironmentIndex {
            environment_uid: gateway_registration_input.env_uid,
        };

        state.borrow_mut().environments.insert_gateway_principal_id_in_env(environment_index,registered_gateway_index.principal_id)?;

        Ok(registered_gateway_value)
    })
}

#[update(name = "getRegisteredGatewaysInEnvironment")]
#[candid_method(update, rename = "getRegisteredGatewaysInEnvironment")]
fn get_registered_gateways_in_environment(environment_uid: EnvironmentUID) -> MultipleRegisteredGatewayResult {
    STATE.with(|state| {
        let environment_index = EnvironmentIndex {
            environment_uid,
        };

        let environment_value = match state.borrow().environments.read(&environment_index) {
            Ok(environment_value) => {
                Ok(environment_value.clone())
            },
            Err(e) => Err(e)
        }?;
        let gateway_principal_ids: Vec<GatewayPrincipalId> = environment_value
            .env_gateways_principals_ids
            .iter()
            .fold(vec![], |mut gateway_principal_ids, (gateway_principal_id, _)| 
        {
            gateway_principal_ids.push(gateway_principal_id.clone());
            gateway_principal_ids
        });

        let mut registered_gateways: Vec<RegisteredGatewayValue> = vec![];
        for gateway_principal_id in gateway_principal_ids {
            let registered_gateway_index = RegisteredGatewayIndex {
                principal_id: gateway_principal_id,
            };
            let registered_gateway_value = match state.borrow().registered_gateways.read(&registered_gateway_index) {
                Ok(registered_gateway_value) => Ok(registered_gateway_value.clone()),
                Err(e) => Err(e),
            }?;
            registered_gateways.push(registered_gateway_value.clone());
        }
        print(format!("{:?}", registered_gateways));
        Ok(registered_gateways)
    })
}