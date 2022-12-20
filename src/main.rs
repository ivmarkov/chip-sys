use chip_sys::{
    callbacks::TestComissionableDataProvider,
    dynamic::{
        Cluster, Clusters, DataVersion, DataVersions, DeviceType, DeviceTypes, Endpoint,
        AGGREGATE_NODE_REGISTRATION,
    },
    *,
};

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

    let _configuration_mgr = unsafe { chip_DeviceLayer_ConfigurationMgr().as_mut() }.unwrap();

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

    let bridged_light = {
        const DEVICE_TYPES: DeviceTypes = &[
            DeviceType::of(DEVICE_TYPE_LO_ON_OFF_LIGHT),
            DeviceType::of(DEVICE_TYPE_BRIDGED_NODE),
        ];
        const DATA_VERSIONS: DataVersions = &[DataVersion::initial(), DataVersion::initial()];
        const CLUSTERS: Clusters = &[Cluster::on_off(), Cluster::descriptor(), Cluster::bridged()];

        Endpoint::new(1, DEVICE_TYPES, DATA_VERSIONS, CLUSTERS)
    };

    let _registration = bridged_light.register(&AGGREGATE_NODE_REGISTRATION);

    println!("Spin loop");

    unsafe {
        platform_mgr.RunEventLoop();
    }

    println!("Exiting");

    Ok(())
}
