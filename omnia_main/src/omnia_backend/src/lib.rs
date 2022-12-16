use ic_cdk::api;

type EnvironmentUID = u32;

#[ic_cdk_macros::import(canister = "database")]
struct Database;



#[ic_cdk_macros::update(name = "createEnvironment")]
async fn create_environment(
    environment_creation_input: EnvironmentCreationInput
) -> Box<EnvironmentCreationResult> {
    let environment_manager_principal = api::caller();

    let environment_creation_result = Database::createNewEnvironment(
        environment_manager_principal.to_string(),
        Box::new(environment_creation_input)
    ).await.0;

    ic_cdk::print(format!("Created new environment: {:?}", environment_creation_result));

    environment_creation_result
}



#[ic_cdk_macros::update(name = "registerGateway")]
async fn register_gateway(
    gateway_registration_input: GatewayRegistrationInput
) -> Box<GatewayRegistrationResult> {

    let environment_manager_principal = api::caller();

    let gateway_registration_result = Database::registerGatewayInEnvironment(
        environment_manager_principal.to_string(),
        Box::new(gateway_registration_input)
    ).await.0;

    ic_cdk::print(format!("Registered gateway: {:?}", gateway_registration_result));

    gateway_registration_result
}



#[ic_cdk_macros::update(name = "registerDevice")]
async fn register_device(
    device_registration_input: DeviceRegistrationInput
) -> Box<DeviceRegistrationResult> {

    let environment_manager_principal = api::caller();

    let device_registration_result = Database::registerDeviceInEnvironment(
        environment_manager_principal.to_string(),
        Box::new(device_registration_input)
    ).await.0;

    ic_cdk::print(format!("Registered device: {:?}", device_registration_result));

    device_registration_result
}



#[ic_cdk_macros::update(name = "getProfile")]
async fn get_profile() -> Box<UserProfile> {
    let user_principal = api::caller();

    let user_profile = Database::getUserProfile(user_principal.to_string()).await.0;

    ic_cdk::print(format!("User profile: {:?}", user_profile));

    user_profile
}



#[ic_cdk_macros::update(name = "setEnvironment")]
async fn set_environment(env_uid: EnvironmentUID) -> Box<EnvironmentInfo> {
    let user_principal = api::caller();

    let environment_info = Database::setUserInEnvironment(user_principal.to_string(), env_uid).await.0;
    ic_cdk::print(format!("User in environment: {:?}", environment_info));

    environment_info
}



#[ic_cdk_macros::update(name = "resetEnvironment")]
async fn reset_environment() -> Box<EnvironmentInfo> {
    let user_principal = api::caller();

    let environment_info = Database::resetUserFromEnvironment(user_principal.to_string()).await.0;
    ic_cdk::print(format!("User not in environment: {:?}", environment_info));

    environment_info
}