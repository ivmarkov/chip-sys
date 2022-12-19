use connectedhomeip_sys::{callbacks::TestComissionableDataProvider, *};

pub fn main() -> Result<(), ChipError> {
    chip!(unsafe { chip_Platform_MemoryInit(core::ptr::null_mut(), 0) })?;

    let platform_mgr = unsafe { chip_DeviceLayer_PlatformMgr().as_mut() }.unwrap();

    chip!(unsafe { platform_mgr.InitChipStack() })?;

    println!("Initialized");

    unsafe {
        glue_Initialize();
    }

    unsafe {
        callbacks::set_comissionable_data_provider(TestComissionableDataProvider);
    }

    unsafe {
        chip_Credentials_SetDeviceAttestationCredentialsProvider(
            chip_Credentials_Examples_GetExampleDACProvider(),
        );
    }

    let init_params = unsafe { glue_CommonCaseDeviceServerInitParams().as_mut() }.unwrap();

    // Init Data Model and CHIP App Server
    chip!(unsafe {
        chip_CommonCaseDeviceServerInitParams_InitializeStaticResourcesBeforeServerInit(
            init_params as *mut _ as *mut _,
        )
    })?;

    let server = unsafe { chip_Server_GetInstance().as_mut() }.unwrap();

    chip!(unsafe { server.Init(init_params as *const _ as *const _) })?;

    let configuration_mgr = unsafe { chip_DeviceLayer_ConfigurationMgr().as_mut() }.unwrap();

    //configuration_mgr.LogDeviceConfig();

    unsafe {
        PrintOnboardingCodes(chip_RendezvousInformationFlags {
            mValue: chip_RendezvousInformationFlag_kOnNetwork,
            _phantom_0: core::marker::PhantomData,
        });
    }

    // /////////////////

    // (taken from chip-devices.xml)
    const DEVICE_TYPE_BRIDGED_NODE: u16 = 0x0013;
    // (taken from lo-devices.xml)
    const DEVICE_TYPE_LO_ON_OFF_LIGHT: u16 = 0x0100;

    // (taken from chip-devices.xml)
    const DEVICE_TYPE_ROOT_NODE: u16 = 0x0016;
    // (taken from chip-devices.xml)
    const DEVICE_TYPE_BRIDGE: u16 = 0x000e;

    // Device Version for dynamic endpoints:
    const DEVICE_VERSION_DEFAULT: u8 = 1;

    static ROOT_DEVICE_TYPES: &'static [EmberAfDeviceType] = &[EmberAfDeviceType {
        deviceId: DEVICE_TYPE_ROOT_NODE,
        deviceVersion: DEVICE_VERSION_DEFAULT,
    }];

    static AGGREGATE_NODE_DEVICE_TYPES: &'static [EmberAfDeviceType] = &[EmberAfDeviceType {
        deviceId: DEVICE_TYPE_BRIDGE,
        deviceVersion: DEVICE_VERSION_DEFAULT,
    }];

    static BRIDGED_ON_OFF_DEVICE_TYPES: &'static [EmberAfDeviceType] = &[
        EmberAfDeviceType {
            deviceId: DEVICE_TYPE_LO_ON_OFF_LIGHT,
            deviceVersion: DEVICE_VERSION_DEFAULT,
        },
        EmberAfDeviceType {
            deviceId: DEVICE_TYPE_BRIDGED_NODE,
            deviceVersion: DEVICE_VERSION_DEFAULT,
        },
    ];

    static mut BRIDGED_LIGHT_CLUSTERS: &'static [EmberAfCluster] = &[
        // EmberAfCluster {

        // }
    ];

    let bridged_light = unsafe {
        EmberAfEndpointType {
            cluster: BRIDGED_LIGHT_CLUSTERS.as_ptr(),
            clusterCount: BRIDGED_LIGHT_CLUSTERS.len() as _,
            endpointSize: 0,
        }
    };

    let bridged_light_data_version: [chip_DataVersion; 0] = [];

    // // DECLARE_DYNAMIC_CLUSTER_LIST_BEGIN(bridgedLightClusters)
    // // DECLARE_DYNAMIC_CLUSTER(ZCL_ON_OFF_CLUSTER_ID, onOffAttrs, onOffIncomingCommands, nullptr),
    // //     DECLARE_DYNAMIC_CLUSTER(ZCL_DESCRIPTOR_CLUSTER_ID, descriptorAttrs, nullptr, nullptr),
    // //     DECLARE_DYNAMIC_CLUSTER(ZCL_BRIDGED_DEVICE_BASIC_CLUSTER_ID, bridgedDeviceBasicAttrs, nullptr,
    // // nullptr) DECLARE_DYNAMIC_CLUSTER_LIST_END;

    // Set starting endpoint id where dynamic endpoints will be assigned, which
    // will be the next consecutive endpoint id after the last fixed endpoint.
    let first_endpoint_id =
        unsafe { emberAfEndpointFromIndex(emberAfFixedEndpointCount() - 1) } + 1;

    // Disable last fixed endpoint, which is used as a placeholder for all of the
    // supported clusters so that ZAP will generated the requisite code.
    unsafe {
        emberAfEndpointEnableDisable(
            emberAfEndpointFromIndex(emberAfFixedEndpointCount() - 1),
            false,
        );
    }

    // A bridge has root node device type on EP0 and aggregate node device type (bridge) at EP1
    unsafe {
        emberAfSetDeviceTypeList(
            0,
            chip_Span {
                mDataBuf: ROOT_DEVICE_TYPES.as_ptr() as *mut _,
                mDataLen: ROOT_DEVICE_TYPES.len(),
                _phantom_0: core::marker::PhantomData,
            },
        );
    }
    unsafe {
        emberAfSetDeviceTypeList(
            1,
            chip_Span {
                mDataBuf: AGGREGATE_NODE_DEVICE_TYPES.as_ptr() as *mut _,
                mDataLen: AGGREGATE_NODE_DEVICE_TYPES.len(),
                _phantom_0: core::marker::PhantomData,
            },
        );
    }

    unsafe {
        emberAfSetDynamicEndpoint(
            0,
            first_endpoint_id,
            &bridged_light,
            &chip_Span {
                mDataBuf: BRIDGED_ON_OFF_DEVICE_TYPES.as_ptr() as *mut _,
                mDataLen: BRIDGED_ON_OFF_DEVICE_TYPES.len(),
                _phantom_0: core::marker::PhantomData,
            },
            chip_Span {
                mDataBuf: bridged_light_data_version.as_ptr() as *mut _,
                mDataLen: bridged_light_data_version.len(),
                _phantom_0: core::marker::PhantomData,
            },
            1,
        );
    }

    // /////////////////

    println!("Spin loop");

    unsafe {
        platform_mgr.RunEventLoop();
    }

    println!("Exiting");

    Ok(())
}
