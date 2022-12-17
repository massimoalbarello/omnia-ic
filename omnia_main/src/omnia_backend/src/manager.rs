use ic_cdk::api;

type EnvironmentUID = u32;

#[ic_cdk_macros::import(canister = "database")]
pub struct Database;



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

    if let EntityRegistrationResult::Gateway(gateway_registration_result) = Database::registerEntity(
        environment_manager_principal.to_string(),
        Box::new(EntityRegistrationInput::Gateway(gateway_registration_input))
    ).await.0 {
        ic_cdk::print(format!("Registered gateway: {:?}", gateway_registration_result));
        gateway_registration_result
    }
    panic!("Should register gateway");
}



#[ic_cdk_macros::update(name = "registerDevice")]
async fn register_device(
    device_registration_input: DeviceRegistrationInput
) -> Box<DeviceRegistrationResult> {

    let environment_manager_principal = api::caller();

    if let EntityRegistrationResult::Device(device_registration_result) = Database::registerEntity(
        environment_manager_principal.to_string(),
        Box::new(EntityRegistrationInput::Device(device_registration_input))
    ).await.0 {
        ic_cdk::print(format!("Registered device: {:?}", device_registration_result));
        device_registration_result
    }
    panic!("Should register device");
}