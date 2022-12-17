use ic_cdk::api::call::ManualReply;
use std::collections::BTreeMap;
use rand::Rng;
use super::interface_types as InterfaceTypes;
use super::store_types as StoreTypes;
use super::ENVIRONMENT_STORE;

type PrincipalId = String;
type EnvironmentUID = u32;



#[ic_cdk_macros::update(name = "createNewEnvironment", manual_reply = true)]
fn create_new_environment(
    environment_manager_principal_id: PrincipalId,
    environment_creation_input: InterfaceTypes::EnvironmentCreationInput
) -> ManualReply<InterfaceTypes::EnvironmentCreationResult> {

    ic_cdk::print(format!("Creating new environment: {:?} managed by: {:?}", environment_creation_input, environment_manager_principal_id));

    let environment_uid = rand::thread_rng().gen_range(0..100);
    ic_cdk::print(format!("Environment UID: {:?}", environment_uid));

    ENVIRONMENT_STORE.with(|environment_store| {
        environment_store.borrow_mut().insert(
            environment_uid,
            StoreTypes::EnvironmentInfo {
                env_name: environment_creation_input.env_name.clone(),
                env_uid: environment_uid,
                env_gateways: BTreeMap::new(),
                env_manager_principal_id: environment_manager_principal_id,
            }
        );
    });

    let environment_creation_result = InterfaceTypes::EnvironmentCreationResult {
        env_name: environment_creation_input.env_name,
        env_uid: environment_uid,
    };

    ManualReply::one(environment_creation_result)
}



#[ic_cdk_macros::update(name = "registerEntity", manual_reply = true)]
fn register_entity(
    principal_id: PrincipalId,
    entity_registration_input: InterfaceTypes::EntityRegistrationInput
) -> ManualReply<InterfaceTypes::EntityRegistrationResult>{
    match get_environment_info_by_uid(&entity_registration_input.getEnvironmentUID()) {
        Some(mut environment_info) => {
            let entity_uid = rand::thread_rng().gen_range(0..100);
            let entity_registration_result = environment_info.update(entity_registration_input, entity_uid);

            ic_cdk::print(format!("Updated environment: {:?}", environment_info));

            ENVIRONMENT_STORE.with(|environment_store| {
                environment_store.borrow_mut().insert(
                    environment_info.env_uid,
                    environment_info
                )
            });

            ManualReply::one(entity_registration_result)
        },
        None => panic!("Environment does not exist"),
    }
}



pub fn get_environment_info_by_uid(environment_uid: &EnvironmentUID) -> Option<StoreTypes::EnvironmentInfo> {
    ENVIRONMENT_STORE.with(|environment_store| {
        match environment_store.borrow().get(environment_uid) {
            Some(mut environment_info) => Some(environment_info.to_owned()),
            None => None,
        }
    })
}
