use candid::candid_method;
use ic_cdk::{export::Principal, print, trap};
use ic_cdk_macros::{update, query};
use omnia_types::environment::{EnvironmentInfoResult, EnvironmentUidIndex, EnvironmentIndex};
use omnia_types::errors::GenericError;
use omnia_types::http::{IpChallengeNonce, IpChallengeIndex};
use omnia_types::virtual_persona::{VirtualPersonaIndex, VirtualPersonaEntry, VirtualPersonaValueResult};
use omnia_types::{
    environment::EnvironmentInfo,
    virtual_persona::{VirtualPersonaValue, VirtualPersonaPrincipalId},
};
use omnia_utils::get_principal_from_string;

use crate::STATE;

#[update(name = "setUserInEnvironment")]
#[candid_method(update, rename = "setUserInEnvironment")]
fn set_user_in_environment(
    virtual_persona_principal_id: VirtualPersonaPrincipalId,
    nonce: IpChallengeNonce,
) -> EnvironmentInfoResult {
    STATE.with(|state| {
        // validate IP challenge
        let ip_challenge_index = IpChallengeIndex {
            nonce,
        };
        let ip_challenge_value = state.borrow_mut().ip_challenges.validate_ip_challenge(&ip_challenge_index)?;

        // update users in environment
        let environment_uid_index = EnvironmentUidIndex {
            ip: ip_challenge_value.requester_ip,
        };
        let environment_uid_value = match state.borrow().environment_uids.read(&environment_uid_index) {
            Ok(environment_uid_value) => Ok(environment_uid_value.clone()),
            Err(e) => Err(e),
        }?;
        let environment_uid = environment_uid_value.env_uid.clone();
        let environment_index = EnvironmentIndex {
            environment_uid: environment_uid_value.env_uid.clone(),
        };
        state.borrow_mut().environments.insert_user_principal_id_in_env(environment_index, virtual_persona_principal_id.clone())?;

        // update environment in virtual persona
        let virtual_persona_index = VirtualPersonaIndex {
            principal_id: virtual_persona_principal_id.clone()
        };
        state.borrow_mut().virtual_personas.insert_env_in_virtual_persona(virtual_persona_index, environment_uid.clone())?;

        print(format!(
            "User: {:?} set in environment with UUID: {:?}",
            virtual_persona_principal_id, environment_uid
        ));

        Ok(EnvironmentInfo {
            env_uid: environment_uid.clone(),
        })
    })
}

#[update(name = "resetUserFromEnvironment")]
#[candid_method(update, rename = "resetUserFromEnvironment")]
fn reset_user_from_environment(virtual_persona_principal_id: VirtualPersonaPrincipalId, nonce: IpChallengeNonce) -> EnvironmentInfoResult {
    STATE.with(|state| {
        // validate IP challenge
        let ip_challenge_index = IpChallengeIndex {
            nonce,
        };
        let ip_challenge_value = state.borrow_mut().ip_challenges.validate_ip_challenge(&ip_challenge_index)?;

        // update users in environment
        let environment_uid_index = EnvironmentUidIndex {
            ip: ip_challenge_value.requester_ip,
        };
        let environment_uid_value = match state.borrow().environment_uids.read(&environment_uid_index) {
            Ok(environment_uid_value) => Ok(environment_uid_value.clone()),
            Err(e) => Err(e),
        }?;
        let environment_uid = environment_uid_value.env_uid.clone();
        let environment_index = EnvironmentIndex {
            environment_uid: environment_uid_value.env_uid.clone(),
        };
        state.borrow_mut().environments.remove_user_principal_id_in_env(environment_index, virtual_persona_principal_id.clone())?;

        // update environment in virtual persona
        let virtual_persona_index = VirtualPersonaIndex {
            principal_id: virtual_persona_principal_id.clone()
        };
        state.borrow_mut().virtual_personas.remove_env_in_virtual_persona(virtual_persona_index)?;

        print(format!(
            "User: {:?} set in environment with UUID: {:?}",
            virtual_persona_principal_id, environment_uid
        ));

        Ok(EnvironmentInfo {
            env_uid: environment_uid.clone(),
        })
    })
}

#[update(name = "getVirtualPersona")]
#[candid_method(update, rename = "getVirtualPersona")]
fn get_virtual_persona(nonce: IpChallengeNonce, virtual_persona_principal_id: VirtualPersonaPrincipalId) -> VirtualPersonaValueResult {
    STATE.with(|state| {
        let ip_challenge_index = IpChallengeIndex {
            nonce,
        };
        let ip_challenge_value = state.borrow_mut().ip_challenges.validate_ip_challenge(&ip_challenge_index)?;
        
        let virtual_persona_index = VirtualPersonaIndex {
            principal_id: virtual_persona_principal_id.clone(),
        };

        // if virtual persona exists, return it
        if let Ok(existing_virtual_persona_value) = state.borrow().virtual_personas.read(&virtual_persona_index) {
            print(format!(
                "User: {:?} has profile: {:?}",
                virtual_persona_index.principal_id, existing_virtual_persona_value
            ));
            return Ok(existing_virtual_persona_value.to_owned());
        }

        // otherwise, create a new one
        let new_virtual_persona_value = VirtualPersonaValue {
            virtual_persona_principal_id,
            virtual_persona_ip: ip_challenge_value.requester_ip,
            user_env_uid: None,
            manager_env_uid: None,
        };

        print(format!(
            "Created profile: {:?} of user: {:?}",
            new_virtual_persona_value, virtual_persona_index.principal_id
        ));

        state.borrow_mut().virtual_personas.create(virtual_persona_index, new_virtual_persona_value.clone()).expect("previous entry should not exist");

        Ok(new_virtual_persona_value)
    })
}

#[query(name = "checkIfVirtualPersonaExists")]
#[candid_method(query, rename = "checkIfVirtualPersonaExists")]
fn check_if_virtual_persona_exists(virtual_persona_principal_id: VirtualPersonaPrincipalId) -> bool {
    let virtual_persona_index = VirtualPersonaIndex {
        principal_id: virtual_persona_principal_id,
    };
    STATE.with(
        |state| match state.borrow_mut().virtual_personas.read(&virtual_persona_index) {
            Ok(_) => true,
            Err(_) => false,
        },
    )
}