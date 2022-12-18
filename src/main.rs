use connectedhomeip_sys::*;

pub fn main() -> Result<(), ChipError> {
    chip!(unsafe { chip_Platform_MemoryInit(core::ptr::null_mut(), 0) })?;

    let platform_mgr = unsafe { chip_DeviceLayer_PlatformMgr().as_mut() }.unwrap();

    chip!(unsafe { platform_mgr.InitChipStack() })?;

    println!("Initialized");

    unsafe {
        glue_InitCommissionableDataProvider();
    }

    unsafe {
        chip_Credentials_SetDeviceAttestationCredentialsProvider(
            chip_Credentials_Examples_GetExampleDACProvider(),
        );
    }

    let init_params = unsafe { glue_chip_CommonCaseDeviceServerInitParams().as_mut() }.unwrap();

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

    // // (taken from chip-devices.xml)
    // const DEVICE_TYPE_BRIDGED_NODE: u16 = 0x0013;
    // // (taken from lo-devices.xml)
    // const DEVICE_TYPE_LO_ON_OFF_LIGHT: u16 = 0x0100;

    // // (taken from chip-devices.xml)
    // const DEVICE_TYPE_ROOT_NODE: u16 = 0x0016;
    // // (taken from chip-devices.xml)
    // const DEVICE_TYPE_BRIDGE: u16 = 0x000e;

    // // Device Version for dynamic endpoints:
    // const DEVICE_VERSION_DEFAULT: u8 = 1;

    // static ROOT_DEVICE_TYPES: &'static [EmberAfDeviceType] = &[EmberAfDeviceType {
    //     deviceId: DEVICE_TYPE_ROOT_NODE,
    //     deviceVersion: DEVICE_VERSION_DEFAULT,
    // }];

    // static AGGREGATE_NODE_DEVICE_TYPES: &'static [EmberAfDeviceType] = &[EmberAfDeviceType {
    //     deviceId: DEVICE_TYPE_BRIDGE,
    //     deviceVersion: DEVICE_VERSION_DEFAULT,
    // }];

    // static BRIDGED_ON_OFF_DEVICE_TYPES: &'static [EmberAfDeviceType] = &[
    //     EmberAfDeviceType {
    //         deviceId: DEVICE_TYPE_LO_ON_OFF_LIGHT,
    //         deviceVersion: DEVICE_VERSION_DEFAULT,
    //     },
    //     EmberAfDeviceType {
    //         deviceId: DEVICE_TYPE_BRIDGED_NODE,
    //         deviceVersion: DEVICE_VERSION_DEFAULT,
    //     },
    // ];

    // static BRIDGED_LIGHT_CLUSTERS: &'static [EmberAfCluster] = &[EmberAfCluster {}];

    // // DECLARE_DYNAMIC_CLUSTER_LIST_BEGIN(bridgedLightClusters)
    // // DECLARE_DYNAMIC_CLUSTER(ZCL_ON_OFF_CLUSTER_ID, onOffAttrs, onOffIncomingCommands, nullptr),
    // //     DECLARE_DYNAMIC_CLUSTER(ZCL_DESCRIPTOR_CLUSTER_ID, descriptorAttrs, nullptr, nullptr),
    // //     DECLARE_DYNAMIC_CLUSTER(ZCL_BRIDGED_DEVICE_BASIC_CLUSTER_ID, bridgedDeviceBasicAttrs, nullptr,
    // // nullptr) DECLARE_DYNAMIC_CLUSTER_LIST_END;

    // // Set starting endpoint id where dynamic endpoints will be assigned, which
    // // will be the next consecutive endpoint id after the last fixed endpoint.
    // let first_endpoint_id = emberAfEndpointFromIndex(emberAfFixedEndpointCount() - 1) + 1;

    // // Disable last fixed endpoint, which is used as a placeholder for all of the
    // // supported clusters so that ZAP will generated the requisite code.
    // emberAfEndpointEnableDisable(
    //     emberAfEndpointFromIndex(emberAfFixedEndpointCount() - 1),
    //     false,
    // );

    // // A bridge has root node device type on EP0 and aggregate node device type (bridge) at EP1
    // unsafe {
    //     glue_emberAfSetDeviceTypeList(0, ROOT_DEVICE_TYPES.as_ptr(), ROOT_DEVICE_TYPES.len());
    // }
    // unsafe {
    //     glue_emberAfSetDeviceTypeList(
    //         1,
    //         AGGREGATE_NODE_DEVICE_TYPES.as_ptr(),
    //         AGGREGATE_NODE_DEVICE_TYPES.len(),
    //     );
    // }

    // // unsafe {
    // //     glue_emberAfSetDynamicEndpoint(
    // //         0,
    // //         first_endpoint_id,
    // //         ep,
    // //         dataVersionStorage,
    // //         BRIDGED_ON_OFF_DEVICE_TYPES.as_ptr(),
    // //         BRIDGED_ON_OFF_DEVICE_TYPES.len(),
    // //         1,
    // //     );
    // // }

    // /////////////////

    println!("Spin loop");

    unsafe {
        platform_mgr.RunEventLoop();
    }

    println!("Exiting");

    Ok(())
}

#[no_mangle]
extern "C" fn gluecb_emberAfActionsClusterInstantActionCallback(
    _command_obj: *mut chip_app_CommandHandler,
    _command_path: *const chip_app_ConcreteCommandPath,
    _command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
) -> bool {
    true
}

#[no_mangle]
extern "C" fn gluecb_MatterActionsPluginServerInitCallback() {}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSetupDiscriminator(
    setup_discriminator: *mut u16,
) -> u16 {
    *unsafe { setup_discriminator.as_mut() }.unwrap() = 3840;

    0
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pIterationCount(
    iteration_count: *mut u32,
) -> u16 {
    *unsafe { iteration_count.as_mut() }.unwrap() = 1000;

    0
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pSalt(
    salt_buf: *mut chip_MutableByteSpan,
) -> u16 {
    static SALT: &'static [u8] = b"SPAKE2P Key Salt";

    //VerifyOrReturnError(saltBuf.size() >= kSpake2p_Max_PBKDF_Salt_Length, CHIP_ERROR_BUFFER_TOO_SMALL);

    unsafe { core::slice::from_raw_parts_mut(glue_MutableByteSpan_data(salt_buf), SALT.len()) }
        .copy_from_slice(SALT);

    unsafe {
        glue_MutableByteSpan_reduce_size(salt_buf, SALT.len());
    }

    0
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pVerifier(
    verifier_buf: *mut chip_MutableByteSpan,
    out_verifier_len: *mut usize,
) -> u16 {
    static VERIFIER: &'static [u8] = &[
        0xB9, 0x61, 0x70, 0xAA, 0xE8, 0x03, 0x34, 0x68, 0x84, 0x72, 0x4F, 0xE9, 0xA3, 0xB2, 0x87,
        0xC3, 0x03, 0x30, 0xC2, 0xA6, 0x60, 0x37, 0x5D, 0x17, 0xBB, 0x20, 0x5A, 0x8C, 0xF1, 0xAE,
        0xCB, 0x35, 0x04, 0x57, 0xF8, 0xAB, 0x79, 0xEE, 0x25, 0x3A, 0xB6, 0xA8, 0xE4, 0x6B, 0xB0,
        0x9E, 0x54, 0x3A, 0xE4, 0x22, 0x73, 0x6D, 0xE5, 0x01, 0xE3, 0xDB, 0x37, 0xD4, 0x41, 0xFE,
        0x34, 0x49, 0x20, 0xD0, 0x95, 0x48, 0xE4, 0xC1, 0x82, 0x40, 0x63, 0x0C, 0x4F, 0xF4, 0x91,
        0x3C, 0x53, 0x51, 0x38, 0x39, 0xB7, 0xC0, 0x7F, 0xCC, 0x06, 0x27, 0xA1, 0xB8, 0x57, 0x3A,
        0x14, 0x9F, 0xCD, 0x1F, 0xA4, 0x66, 0xCF,
    ];
    //static VERIFIER: &'static [u8] = b"uWFwqugDNGiEck/po7KHwwMwwqZgN10XuyBajPGuyzUEV/iree4lOrao5GuwnlQ65CJzbeUB49s31EH+NEkg0JVI5MGCQGMMT/SRPFNRODm3wH/MBiehuFc6FJ/NH6Rmzw==";

    if unsafe { glue_MutableByteSpan_size(verifier_buf) } < VERIFIER.len() {
        panic!("!!!");
    }

    unsafe {
        core::slice::from_raw_parts_mut(glue_MutableByteSpan_data(verifier_buf), VERIFIER.len())
    }
    .copy_from_slice(VERIFIER);

    unsafe {
        glue_MutableByteSpan_reduce_size(verifier_buf, VERIFIER.len());
    }

    *unsafe { out_verifier_len.as_mut() }.unwrap() = VERIFIER.len();

    0
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSetupPasscode(setup_passcode: *mut u32) -> u16 {
    *unsafe { setup_passcode.as_mut() }.unwrap() = 20202021;

    0
}

extern "C" {
    fn glue_InitCommissionableDataProvider();

    fn glue_MutableByteSpan_data(span: *mut chip_MutableByteSpan) -> *mut u8;
    fn glue_MutableByteSpan_size(span: *mut chip_MutableByteSpan) -> usize;
    fn glue_MutableByteSpan_reduce_size(span: *mut chip_MutableByteSpan, size: usize);

    fn glue_chip_CommonCaseDeviceServerInitParams() -> *mut chip_CommonCaseDeviceServerInitParams;

    //     fn glue_emberAfSetDeviceTypeList(
    //         endpoint: chip::EndpointId,
    //         device_type_list: *const EmberAfDeviceType,
    //         device_type_list_len: usize,
    //     ) -> u32;
    //     fn glue_emberAfSetDynamicEndpoint(
    //         index: u16,
    //         id: chip::EndpointId,
    //         ep: *const EmberAfEndpointType,
    //         data_version_storage: *const chip::DataVersion,
    //         data_version_storage_len: usize,
    //         device_type_list: *const EmberAfDeviceType,
    //         device_type_list_len: usize,
    //         parentEndpointId: chip::EndpointId,
    //     ) -> EmberAfStatus;
}
