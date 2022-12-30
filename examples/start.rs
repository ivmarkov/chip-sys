use chip_sys::{
    callbacks::TestComissionableDataProvider,
    dynamic::{
        initialize, Cluster, Clusters, DeviceType, DeviceTypes, Endpoint, BRIDGE_NODE,
        ENDPOINT_ID_RANGE_START,
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
        callbacks::initialize(None, None, None, Some(&TestComissionableDataProvider));
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

    initialize()?;

    println!("Endpoints initialized");

    let mut data_versions = [0; 3];

    let _registration = LIGHT.register(&mut data_versions, &BRIDGE_NODE).unwrap();

    println!("Spin loop");

    unsafe {
        platform_mgr.RunEventLoop();
    }

    println!("Exiting");

    Ok(())
}
