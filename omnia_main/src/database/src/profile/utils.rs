use ic_cdk::api::call::ManualReply;
use ic_cdk::export::Principal;
use super::USER_PROFILE_STORE;
use super::store_types as ProfileStoreTypes;
use super::super::environment::{
    store_types as EnvironmentStoreTypes,
    utils as EnvironmentUtils
};

type PrincipalId = String;
type EnvironmentUID = u32;



#[ic_cdk_macros::update(name = "setUserInEnvironment", manual_reply = true)]
fn set_user_in_environment(user_principal_id: PrincipalId, env_uid: EnvironmentUID) -> ManualReply<EnvironmentStoreTypes::EnvironmentInfo> {

    let user_principal = Principal::from_text(user_principal_id).unwrap();

    match get_user_profile_if_exists(user_principal) {
        Some(user_profile) => {
            match EnvironmentUtils::get_environment_info_by_uid(&env_uid) {
                Some(environment_info) => {
                    let updated_user_profile = ProfileStoreTypes::UserProfile {
                        environment_uid : Some(env_uid.to_owned()),
                        ..user_profile
                    };

                    USER_PROFILE_STORE.with(|profile_store| {
                        profile_store.borrow_mut().insert(user_principal, updated_user_profile);
                    });

                    ic_cdk::print(format!("User: {:?} set in environment: {:?}", user_principal, environment_info));

                    ManualReply::one(environment_info)
                },
                None => panic!("Environment does not exist"),
            }
        },
        None => panic!("User does not have a profile")
    }
}



#[ic_cdk_macros::update(name = "resetUserFromEnvironment", manual_reply = true)]
fn reset_user_from_environment(user_principal_id: PrincipalId) -> ManualReply<EnvironmentStoreTypes::EnvironmentInfo> {

    let user_principal = Principal::from_text(user_principal_id).unwrap();

    match get_user_profile_if_exists(user_principal) {
        Some(user_profile) => {
            let updated_user_profile = ProfileStoreTypes::UserProfile {
                environment_uid : None,
                ..user_profile
            };

            USER_PROFILE_STORE.with(|profile_store| {
                profile_store.borrow_mut().insert(user_principal, updated_user_profile)
            });

            match user_profile.environment_uid {
                Some(old_environment_uid) => {
                    match EnvironmentUtils::get_environment_info_by_uid(&old_environment_uid) {
                        Some(environment_info) => ManualReply::one(environment_info),
                        None => panic!("Environment does not exist"),
                    }
                },
                None => panic!("User is not in environment"),
            }
        },
        None => panic!("User does not have a profile")
    }
}



#[ic_cdk_macros::update(name = "getUserProfile", manual_reply = true)]
fn get_user_profile(user_principal_id: PrincipalId) -> ManualReply<ProfileStoreTypes::UserProfile> {

    let user_principal = Principal::from_text(user_principal_id).unwrap();

    match get_user_profile_if_exists(user_principal) {
        Some(user_profile) => {
            ic_cdk::print(format!("User: {:?} has profile: {:?}", user_principal, user_profile));
            ManualReply::one(user_profile)
        },
        None => {
            ic_cdk::print("User does not have a profile");

            // create new user profile
            let new_user_profile = ProfileStoreTypes::UserProfile {
                user_principal_id: user_principal.to_string(),
                environment_uid: None,
            };

            ic_cdk::print(format!("Created profile: {:?} of user: {:?}", new_user_profile, user_principal));

            USER_PROFILE_STORE.with(|profile_store| {
                profile_store.borrow_mut().insert(user_principal, new_user_profile.clone());
            });

            ManualReply::one(new_user_profile)
        }
    }
}



fn get_user_profile_if_exists(user_principal: Principal) -> Option<ProfileStoreTypes::UserProfile> {
    USER_PROFILE_STORE.with(|profile_store| {
        match profile_store.borrow().get(&user_principal) {
            Some(user_profile) => Some(user_profile.to_owned()),
            None => None
        }
    })
}