use x509_parser::pem::parse_x509_pem;
use x509_parser::parse_x509_certificate;
use candid::candid_method;
use ic_cdk::{
    api::{call::call, caller},
    print,
};
use ic_cdk_macros::update;
use omnia_types::{
    environment::{EnvironmentCreationInput, EnvironmentCreationResult, EnvironmentUID},
    gateway::{RegisteredGatewayResult, GatewayRegistrationInput, MultipleRegisteredGatewayResult, InitializedGatewayValue, GatewayPrincipalId}, http::IpChallengeNonce, errors::{GenericResult, GenericError}
};

use crate::utils::get_database_principal;

#[update(name = "createEnvironment")]
#[candid_method(update, rename = "createEnvironment")]
async fn create_environment(
    environment_creation_input: EnvironmentCreationInput,
) -> GenericResult<EnvironmentCreationResult> {
    let environment_manager_principal_id = caller().to_string();

    let (virtual_persona_exists, ): (bool, ) = call(
        get_database_principal(),
        "checkIfVirtualPersonaExists",
        (environment_manager_principal_id.clone(),),
    ).await.unwrap();
    match virtual_persona_exists {
        true => {
            let (environment_creation_result,): (Result<EnvironmentCreationResult, GenericError>,) = call(
                get_database_principal(),
                "createNewEnvironment",
                (
                    environment_manager_principal_id,
                    Box::new(environment_creation_input),
                ),
            )
            .await
            .unwrap();
        
            print(format!(
                "Created new environment: {:?}",
                environment_creation_result
            ));
        
            environment_creation_result
        },
        false => {
            let err = format!(
                "Virtual persona with principal id: {:?} does not exist",
                environment_manager_principal_id
            );

            println!("{}", err);
            Err(err)
        }
    }
}

#[update(name = "initGateway")]
#[candid_method(update, rename = "initGateway")]
async fn init_gateway(nonce: IpChallengeNonce) -> GenericResult<GatewayPrincipalId> {
    let gateway_principal_id = caller().to_string();

    match call(get_database_principal(), "initGatewayByIp", (
        nonce,
        gateway_principal_id,
    ))
    .await
    .unwrap() {
        (Ok(principal_id),) => {
            print(format!("Initialized gateway with prinipal ID: {:?}", principal_id));
            Ok(principal_id)
        },
        (Err(e),) => Err(e)
    }
}

#[update(name = "getInitializedGateways")]
#[candid_method(update, rename = "getInitializedGateways")]
async fn get_initialized_gateways(nonce: IpChallengeNonce) -> GenericResult<Vec<InitializedGatewayValue>> {
    
    let initialized_gateway_principals_result: GenericResult<Vec<InitializedGatewayValue>> = match call(get_database_principal(), "getInitializedGatewaysByIp", (nonce, ))
        .await
        .unwrap()
    {
        (Ok(initialized_gateway_principals),) => {
            print(format!("Initialized gateways in the local network have principals {:?}", initialized_gateway_principals));
            Ok(initialized_gateway_principals)
        },
        (Err(e),) => Err(e)
    };
    initialized_gateway_principals_result
}

#[update(name = "registerGateway")]
#[candid_method(update, rename = "registerGateway")]
async fn register_gateway(
    nonce: IpChallengeNonce,
    gateway_registration_input: GatewayRegistrationInput,
) -> RegisteredGatewayResult {
    let environment_manager_principal = caller();

    let (gateway_registration_result,): (RegisteredGatewayResult,) = call(
        get_database_principal(),
        "registerGatewayInEnvironment",
        (
            nonce,
            environment_manager_principal.to_string(),
            Box::new(gateway_registration_input),
        ),
    )
    .await
    .unwrap();

    gateway_registration_result
}

#[update(name = "getRegisteredGateways")]
#[candid_method(update, rename = "getRegisteredGateways")]
async fn get_registered_gateways(environment_uid: EnvironmentUID) -> MultipleRegisteredGatewayResult {
    let (res,): (MultipleRegisteredGatewayResult,) = call(
        get_database_principal(),
        "getRegisteredGatewaysInEnvironment",
        (environment_uid.clone(),),
    )
    .await
    .unwrap();

    res
}

#[update(name = "parseDeviceCertificate")]
#[candid_method(update, rename = "parseDeviceCertificate")]
async fn parse_device_certificate() {
    let cert_str = "-----BEGIN CERTIFICATE-----\nMIICDTCCAbKgAwIBAgIQe3eNNaVHZutrY7gRg4ItsjAKBggqhkjOPQQDAjBTMQsw\nCQYDVQQGEwJVUzEXMBUGA1UEChMORGlnaUNlcnQsIEluYy4xKzApBgNVBAMTIkRp\nZ2lDZXJ0IFJvb3QgQ0EgZm9yIE1BVFRFUiBQS0kgRzEwIBcNMjIwODI0MDAwMDAw\nWhgPOTk5OTEyMzEyMzU5NTlaMFMxCzAJBgNVBAYTAlVTMRcwFQYDVQQKEw5EaWdp\nQ2VydCwgSW5jLjErMCkGA1UEAxMiRGlnaUNlcnQgUm9vdCBDQSBmb3IgTUFUVEVS\nIFBLSSBHMTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABAVbq6wD9zzDXbEObnSN\nOMNLrGyLBok/Le7bYMzRBn8G4aNSEDw1ClO4gAbrZqpDJy5QSmF9VpKPx9FOsvmV\nbZujZjBkMBIGA1UdEwEB/wQIMAYBAf8CAQEwDgYDVR0PAQH/BAQDAgEGMB0GA1Ud\nDgQWBBQyUEUZM0RZm0Zl1Fn9OhXxwRbMvTAfBgNVHSMEGDAWgBQyUEUZM0RZm0Zl\n1Fn9OhXxwRbMvTAKBggqhkjOPQQDAgNJADBGAiEAh88I/wwZ6/x4wrLLZeEZZEQi\nKqmgvTeRD3kPQ1LoCFgCIQCKVfavo16G+mSmMEFD2O/vsx15c2U1SS0rTK/ogRAP\n4g==\n-----END CERTIFICATE-----";

    let cert_bytes = cert_str.as_bytes();
    let res = parse_x509_pem(cert_bytes);
    if let Ok((_rem, pem)) = res {
        let res_x509 = parse_x509_certificate(&pem.contents);
        print(format!("\nX509 certificate: {:?}", res_x509));
    }
}