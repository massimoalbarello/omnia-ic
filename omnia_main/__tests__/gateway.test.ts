import { DevicesAccessInfo, EnvironmentCreationResult, RegisteredDeviceIndex, RegisteredGatewayValue, UpdateValue } from "../src/declarations/omnia_backend/omnia_backend.did";
import {
  gateway1,
  gateway1Data,
  gateway2,
  gateway2Data,
  manager1,
  manager1Data,
  manager2,
  manager2Data,
} from "./utils/actors";
import { DEVICE_AFFORDANCE_VALUE, DEVICE_PAIRING_PAYLOAD, ENVIRONMENT_NAME, GATEWAY1_NAME, LONG_TEST_TIMEOUT, OMNIA_PROXY_IPV4 } from "./utils/constants";

let environmentUid: string;
let deviceUid: string;

// every test takes a long time
jest.setTimeout(LONG_TEST_TIMEOUT);

describe("Gateway", () => {
  it("initGateway: any Gateway can initialize itself", async () => {
    // this is a proxied gateway
    const gateway1Actor = await gateway1.getActor();
    const gateway1Init = await gateway1.callMethodWithChallenge(
      async (nonce) => {
        return gateway1Actor.initGateway(nonce);
      },
      gateway1Data.remoteIp,
      gateway1Data.proxyData,
    );
    expect(gateway1Init.error).toBeNull();
    expect(gateway1Init.data).toEqual((await gateway1Data.identity).getPrincipal().toText());

    // this is a gateway that exposes itself on the Internet
    const gateway2Actor = await gateway2.getActor();
    const gateway2Init = await gateway2.callMethodWithChallenge(
      async (nonce) => {
        return gateway2Actor.initGateway(nonce);
      },
      gateway2Data.remoteIp,
      gateway2Data.proxyData,
    );
    expect(gateway2Init.error).toBeNull();
    expect(gateway2Init.data).toEqual((await gateway2Data.identity).getPrincipal().toText());
  }, LONG_TEST_TIMEOUT);

  it("getRegisteredDevices: Gateway can retrieve the list of registered devices, empty", async () => {
    const gateway1Actor = await gateway1.getActor();
    const registeredDevicesResult = await gateway1.parseResult(
      gateway1Actor.getRegisteredDevices()
    );
    expect(registeredDevicesResult.error).toBeNull();
    expect(registeredDevicesResult.data).toEqual([]);
  });

  it("createEnvironment: Manager can create an environment", async () => {
    const manager1Actor = await manager1.getActor();
    // just ensure the Manager has a profile initialized
    await manager1.callMethodWithChallenge(
      async (nonce) => {
        return manager1Actor.getProfile(nonce);
      },
      manager1Data.remoteIp,
    );

    // create the environment
    const createEnvironmentResult = await manager1.parseResult(
      manager1Actor.createEnvironment({
        env_name: ENVIRONMENT_NAME,
      })
    );
    expect(createEnvironmentResult.error).toBeNull();
    expect(createEnvironmentResult.data).toMatchObject<EnvironmentCreationResult>({
      env_uid: expect.any(String),
      env_name: ENVIRONMENT_NAME,
    });

    environmentUid = createEnvironmentResult.data!.env_uid;
  });

  it("getInitializedGateways: Manager can retrieve the list of initialized Gateways under its environment", async () => {
    const manager1Actor = await manager1.getActor();
    const initializedGatewaysResult = await manager1.callMethodWithChallenge(
      async (nonce) => {
        return manager1Actor.getInitializedGateways(nonce);
      },
      manager1Data.remoteIp,
    );
    expect(initializedGatewaysResult.error).toBeNull();
    expect(initializedGatewaysResult.data).toEqual([{
      principal_id: (await gateway1Data.identity).getPrincipal().toText(),
    }]);
  });

  it("registerGateway: another Manager cannot register the Gateway in the environment", async () => {
    const manager2Actor = await manager2.getActor();

    // we have to be sure that the Manager and the Gateway are in the same environment
    // this test should always pass
    expect(manager2Data.remoteIp).not.toEqual(gateway1Data.remoteIp);

    // register the Gateway
    const registerGatewayResult = await manager2.callMethodWithChallenge(
      async (nonce) => {
        return manager2Actor.registerGateway(
          nonce,
          {
            gateway_name: GATEWAY1_NAME,
            env_uid: environmentUid,
          }
        );
      },
      manager2Data.remoteIp,
    );
    expect(registerGatewayResult.error).toBeTruthy();
    expect(registerGatewayResult.data).toBeNull();
  });

  it("registerGateway: Manager can register the Gateway in the environment", async () => {
    const manager1Actor = await manager1.getActor();

    // we have to be sure that the Manager and the Gateway are in the same environment
    // this test should always pass
    expect(manager1Data.remoteIp).toEqual(gateway1Data.remoteIp);

    // register the Gateway
    const registerGatewayResult = await manager1.callMethodWithChallenge(
      async (nonce) => {
        return manager1Actor.registerGateway(
          nonce,
          {
            gateway_name: GATEWAY1_NAME,
            env_uid: environmentUid,
          }
        );
      },
      manager1Data.remoteIp,
    );
    expect(registerGatewayResult.error).toBeNull();
    expect(registerGatewayResult.data).toMatchObject<RegisteredGatewayValue>({
      env_uid: environmentUid,
      gateway_name: GATEWAY1_NAME,
      gat_registered_device_uids: [],
      gateway_ip: manager1Data.remoteIp,
      // since the Gateway is proxied, the gateway_url is the proxy's IP
      gateway_url: `https://${OMNIA_PROXY_IPV4}`,
      proxied_gateway_uid: [gateway1Data.proxyData!.peerId],
    });
  });

  it("getGatewayUpdates: Gateway can poll for updates, empty", async () => {
    const gateway1Actor = await gateway1.getActor();
    const gatewayUpdates = await gateway1Actor.getGatewayUpdates();
    expect(gatewayUpdates).toEqual([]);
  });

  it("pairNewDevice: Manager can send the pair command for a new device", async () => {
    const manager1Actor = await manager1.getActor();
    const pairNewDeviceResult = await manager1.callMethodWithChallenge(
      async (nonce) => {
        return manager1Actor.pairNewDevice(
          nonce,
          (await gateway1Data.identity).getPrincipal().toText(),
          DEVICE_PAIRING_PAYLOAD,
        );
      },
      manager1Data.remoteIp,
    );
    expect(pairNewDeviceResult.error).toBeNull();
    expect(pairNewDeviceResult.data).toMatchObject<UpdateValue>({
      virtual_persona_principal_id: (await manager1Data.identity).getPrincipal().toText(),
      virtual_persona_ip: manager1Data.remoteIp,
      command: "pair",
      info: {
        payload: DEVICE_PAIRING_PAYLOAD,
      },
    });
  });

  it("getGatewayUpdates: Gateway can poll for updates, update received", async () => {
    const gateway1Actor = await gateway1.getActor();
    const gatewayUpdates = await gateway1Actor.getGatewayUpdates();
    expect(gatewayUpdates).toEqual([{
      virtual_persona_principal_id: (await manager1Data.identity).getPrincipal().toText(),
      virtual_persona_ip: manager1Data.remoteIp,
      command: "pair",
      info: {
        payload: DEVICE_PAIRING_PAYLOAD,
      },
    }]);
  });

  // here we assume the gateway pairs the new device

  it("registerDevice: Gateway can register the new device paired", async () => {
    const gateway1Actor = await gateway1.getActor();
    const registerDeviceResult = await gateway1.callMethodWithChallenge(
      async (nonce) => {
        return gateway1Actor.registerDevice(
          nonce,
          [
            DEVICE_AFFORDANCE_VALUE,
          ]
        );
      },
      gateway1Data.remoteIp,
      gateway1Data.proxyData,
    );
    expect(registerDeviceResult.error).toBeNull();
    expect(registerDeviceResult.data).toMatchObject<RegisteredDeviceIndex>({
      device_uid: expect.any(String),
    });

    deviceUid = registerDeviceResult.data!.device_uid;
  });

  it("getRegisteredDevices: Gateway can retrieve the list of registered devices, device present", async () => {
    const gateway1Actor = await gateway1.getActor();
    const registeredDevicesResult = await gateway1.parseResult(
      gateway1Actor.getRegisteredDevices()
    );
    expect(registeredDevicesResult.error).toBeNull();
    expect(registeredDevicesResult.data).toEqual([
      deviceUid,
    ]);
  });

  it("getDevicesInEnvironmentByAffordance: Manager (should be an Application) can retrieve the devices by affordances", async () => {
    const manager1Actor = await manager1.getActor();
    const devicesInEnvironmentByAffordanceResult = await manager1.parseResult(
      manager1Actor.getDevicesInEnvironmentByAffordance(
        environmentUid,
        DEVICE_AFFORDANCE_VALUE,
      )
    );
    expect(devicesInEnvironmentByAffordanceResult.error).toBeNull();
    expect(devicesInEnvironmentByAffordanceResult.data).toMatchObject<DevicesAccessInfo>({
      devices_urls: [
        `https://${OMNIA_PROXY_IPV4}/${deviceUid}`,
      ],
      required_headers: [
        ["x-forward-to-peer", gateway1Data.proxyData!.peerId,],
        ["x-forward-to-port", "8888"],
      ],
    });

    const emptyDevicesInEnvironmentByAffordanceResult = await manager1.parseResult(
      manager1Actor.getDevicesInEnvironmentByAffordance(
        environmentUid,
        "non-existent-affordance",
      )
    );
    expect(emptyDevicesInEnvironmentByAffordanceResult.error).toBeNull();
    expect(emptyDevicesInEnvironmentByAffordanceResult.data).toEqual([]);
  });
});
