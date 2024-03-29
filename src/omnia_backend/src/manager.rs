use candid::candid_method;
use ic_cdk::{
    api::{call::call, caller},
    print, trap,
};
use ic_cdk_macros::update;
use ic_oxigraph::model::{vocab, GraphName, Literal, NamedNode, Quad};
use omnia_types::{
    device::{DeviceAffordances, RegisteredDeviceResult, RegisteredDevicesUidsResult},
    environment::{EnvironmentCreationInput, EnvironmentCreationResult, EnvironmentUID},
    errors::{GenericError, GenericResult},
    gateway::{
        GatewayPrincipalId, GatewayRegistrationInput, InitializedGatewayValue,
        MultipleRegisteredGatewayResult, RegisteredGatewayResult,
    },
    http::IpChallengeNonce,
    updates::{PairingPayload, UpdateValueOption, UpdateValueResult},
    virtual_persona::VirtualPersonaPrincipalId,
};

use crate::{
    rdf::{BotNode, HttpNode, OmniaNode, SarefNode, TdNode, UrnNode},
    utils::get_database_principal,
    RDF_DB,
};

#[update(name = "createEnvironment")]
#[candid_method(update, rename = "createEnvironment")]
async fn create_environment(
    environment_creation_input: EnvironmentCreationInput,
) -> GenericResult<EnvironmentCreationResult> {
    let environment_manager_principal_id = caller().to_string();

    let (virtual_persona_exists,): (bool,) = call(
        get_database_principal(),
        "checkIfVirtualPersonaExists",
        (environment_manager_principal_id.clone(),),
    )
    .await
    .unwrap();
    match virtual_persona_exists {
        true => {
            let (environment_creation_result,): (Result<EnvironmentCreationResult, GenericError>,) =
                call(
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

            // register the environment in the RDF database
            match environment_creation_result {
                Ok(result) => {
                    let quad = Quad::new(
                        UrnNode::new_uuid(&result.env_uid),
                        vocab::rdf::TYPE,
                        BotNode::new("Zone"),
                        GraphName::DefaultGraph,
                    );

                    RDF_DB.with(|rdf_db| match rdf_db.borrow().insert(&quad) {
                        Ok(_) => Ok(result),
                        Err(err) => Err(format!("Error inserting quad: {:?}", err)),
                    })
                }
                Err(err) => Err(err),
            }
        }
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

    let is_registered = call::<(GatewayPrincipalId,), (bool,)>(
        get_database_principal(),
        "isGatewayRegistered",
        (gateway_principal_id.clone(),),
    )
    .await
    .unwrap()
    .0;

    if !is_registered {
        print(format!(
            "Gateway with principal ID: {:?} is not yet registered",
            gateway_principal_id
        ));
        let principal_id =
            call::<(IpChallengeNonce, GatewayPrincipalId), (GenericResult<GatewayPrincipalId>,)>(
                get_database_principal(),
                "initGatewayByIp",
                (nonce, gateway_principal_id),
            )
            .await
            .unwrap()
            .0?;

        return Ok(principal_id);
    }
    Ok(gateway_principal_id)
}

#[update(name = "getInitializedGateways")]
#[candid_method(update, rename = "getInitializedGateways")]
async fn get_initialized_gateways(
    nonce: IpChallengeNonce,
) -> GenericResult<Vec<InitializedGatewayValue>> {
    let initialized_gateway_principals_result: GenericResult<Vec<InitializedGatewayValue>> =
        match call(
            get_database_principal(),
            "getInitializedGatewaysByIp",
            (nonce,),
        )
        .await
        .unwrap()
        {
            (Ok(initialized_gateway_principals),) => {
                print(format!(
                    "Initialized gateways in the local network have principals {:?}",
                    initialized_gateway_principals
                ));
                Ok(initialized_gateway_principals)
            }
            (Err(e),) => Err(e),
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
async fn get_registered_gateways(
    environment_uid: EnvironmentUID,
) -> MultipleRegisteredGatewayResult {
    let (res,): (MultipleRegisteredGatewayResult,) = call(
        get_database_principal(),
        "getRegisteredGatewaysInEnvironment",
        (environment_uid.clone(),),
    )
    .await
    .unwrap();

    res
}

#[update(name = "getGatewayUpdates")]
#[candid_method(update, rename = "getGatewayUpdates")]
async fn get_gateway_updates() -> UpdateValueOption {
    let gateway_principal_id = caller().to_string();

    call::<(GatewayPrincipalId,), (UpdateValueOption,)>(
        get_database_principal(),
        "getGatewayUpdatesByPrincipal",
        (gateway_principal_id,),
    )
    .await
    .unwrap()
    .0
}

#[update(name = "pairNewDevice")]
#[candid_method(update, rename = "pairNewDevice")]
async fn pair_new_device(
    nonce: IpChallengeNonce,
    gateway_principal_id: GatewayPrincipalId,
    pairing_payload: PairingPayload,
) -> UpdateValueResult {
    let manager_principal_id = caller().to_string();

    call::<
        (
            IpChallengeNonce,
            VirtualPersonaPrincipalId,
            GatewayPrincipalId,
            PairingPayload,
        ),
        (UpdateValueResult,),
    >(
        get_database_principal(),
        "pairNewDeviceOnGateway",
        (
            nonce,
            manager_principal_id,
            gateway_principal_id,
            pairing_payload,
        ),
    )
    .await
    .unwrap()
    .0
}

#[update(name = "registerDevice")]
#[candid_method(update, rename = "registerDevice")]
async fn register_device(
    nonce: IpChallengeNonce,
    affordances: DeviceAffordances,
) -> RegisteredDeviceResult {
    let gateway_principal_id = caller().to_string();

    let registered_device =
        call::<(IpChallengeNonce, GatewayPrincipalId), (RegisteredDeviceResult,)>(
            get_database_principal(),
            "registerDeviceOnGateway",
            (nonce, gateway_principal_id),
        )
        .await
        .unwrap()
        .0?;

    let device_url = registered_device.clone().1.device_url.clone();

    let device_node = NamedNode::new(device_url.clone()).map_err(|err| {
        format!(
            "Error while creating device node for device with URL: {:?} {:?}",
            device_url, err
        )
    })?;

    let mut quads: Vec<Quad> = vec![
        // device declaration
        Quad::new(
            device_node.clone(),
            vocab::rdf::TYPE,
            SarefNode::new("Device"),
            GraphName::DefaultGraph,
        ),
        // device - environment relation
        Quad::new(
            UrnNode::new_uuid(&registered_device.clone().1.env_uid),
            BotNode::new("hasElement"),
            device_node.clone(),
            GraphName::DefaultGraph,
        ),
    ];

    // device required HTTP headers
    // TODO: define better names for HTTP headers
    let required_headers = registered_device.clone().1.required_headers;
    if required_headers.is_some() {
        required_headers.unwrap().iter().enumerate().for_each(
            |(i, (header_name, header_value))| {
                let header_node = OmniaNode::new(&format!("HTTPHeader{}", i));

                quads.extend_from_slice(&[
                    Quad::new(
                        header_node.clone(),
                        vocab::rdf::TYPE,
                        HttpNode::new("RequestHeader"),
                        GraphName::DefaultGraph,
                    ),
                    Quad::new(
                        header_node.clone(),
                        HttpNode::new("fieldName"),
                        Literal::new_simple_literal(header_name),
                        GraphName::DefaultGraph,
                    ),
                    Quad::new(
                        header_node.clone(),
                        HttpNode::new("fieldValue"),
                        Literal::new_simple_literal(header_value),
                        GraphName::DefaultGraph,
                    ),
                    Quad::new(
                        device_node.clone(),
                        OmniaNode::new("requiresHeader"),
                        header_node,
                        GraphName::DefaultGraph,
                    ),
                ]);
            },
        );
    }

    quads.extend(affordances.properties.iter().map(|affordance| {
        Quad::new(
            device_node.clone(),
            TdNode::new("hasPropertyAffordance"),
            SarefNode::from_string(&affordance),
            GraphName::DefaultGraph,
        )
    }));

    quads.extend(affordances.actions.iter().map(|affordance| {
        Quad::new(
            device_node.clone(),
            TdNode::new("hasActionAffordance"),
            SarefNode::from_string(&affordance),
            GraphName::DefaultGraph,
        )
    }));

    // TODO: handle outcall errors. For example we may want to retry or remove the registered device
    quads.iter().for_each(|quad| {
        RDF_DB.with(|rdf_db| match rdf_db.borrow().insert(quad) {
            Ok(_) => (),
            Err(e) => {
                trap(&format!("Error inserting quad: {}", e));
            }
        });
    });

    Ok(registered_device)
}

#[update(name = "getRegisteredDevices")]
#[candid_method(update, rename = "getRegisteredDevices")]
async fn get_registered_devices() -> RegisteredDevicesUidsResult {
    let gateway_principal_id = caller().to_string();

    call::<(GatewayPrincipalId,), (RegisteredDevicesUidsResult,)>(
        get_database_principal(),
        "getRegisteredDevicesOnGateway",
        (gateway_principal_id,),
    )
    .await
    .unwrap()
    .0
}
