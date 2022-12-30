use chip_sys::{
    chip::{
        Cluster, Clusters, DeviceType, DeviceTypes, Endpoint, TestComissionableDataProvider,
        ENDPOINT_ID_RANGE_START, EP_0, EP_1, EP_2,
    },
    *,
};

static LIGHT: &Endpoint<'static, 'static> = {
    const DEVICE_TYPES: DeviceTypes = &[
        DeviceType::of(0x0100), // taken from lo-devices.xml
        DeviceType::of(0x0013), // taken from chip-devices.xml
    ];
    const CLUSTERS: Clusters = &[Cluster::on_off(), Cluster::descriptor(), Cluster::bridged()];

    &Endpoint::new(ENDPOINT_ID_RANGE_START, DEVICE_TYPES, CLUSTERS)
};

pub fn main() -> Result<(), ChipError> {
    chip!(unsafe { chip_Platform_MemoryInit(core::ptr::null_mut(), 0) })?;

    let platform_mgr = unsafe { chip_DeviceLayer_PlatformMgr().as_mut() }.unwrap();

    chip!(unsafe { platform_mgr.InitChipStack() })?;

    println!("Initialized");

    unsafe {
        cb::initialize(None, None, None, Some(&TestComissionableDataProvider));
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

    let _configuration_mgr = unsafe { chip_DeviceLayer_ConfigurationMgr().as_mut() }.unwrap();

    //configuration_mgr.LogDeviceConfig();

    unsafe {
        PrintOnboardingCodes(chip_RendezvousInformationFlags {
            mValue: chip_RendezvousInformationFlag_kOnNetwork,
            _phantom_0: core::marker::PhantomData,
        });
    }

    // /////////////////

    // Disable last fixed endpoint, which is used as a placeholder for all of the
    // supported clusters so that ZAP will generate the requisite code.
    EP_2.enable(false);

    //
    // A bridge has root node device type on EP0 and aggregate node device type (bridge) at EP1
    //

    static ROOT_DEVICE_TYPES: &[DeviceType] = &[DeviceType::of(0x0016)]; // taken from chip-devices.xml
    EP_0.initialize(ROOT_DEVICE_TYPES)?;

    static BRIDGE_NODE_DEVICE_TYPES: &[DeviceType] = &[DeviceType::of(0x000e)]; // taken from chip-devices.xml
    EP_1.initialize(BRIDGE_NODE_DEVICE_TYPES)?;

    println!("Endpoints initialized");

    let mut data_versions = [0; 3];

    let _registration = LIGHT.register(&mut data_versions, EP_1).unwrap();

    println!("Spin loop");

    unsafe {
        platform_mgr.RunEventLoop();
    }

    println!("Exiting");

    Ok(())
}
