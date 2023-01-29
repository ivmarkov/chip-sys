// TODO: Probably belongs to `chip-rs` or suchlike separate crate

use core::borrow::Borrow;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::{ptr, slice};

use crate::*;

static CTX_TAKEN: AtomicBool = AtomicBool::new(false);

pub struct ChipContext(bool, PhantomData<*const ()>);

impl ChipContext {
    pub fn take() -> Result<Self, ChipError> {
        if let Ok(false) =
            CTX_TAKEN.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(Self(true, PhantomData))
        } else {
            Err(ChipError::from_code(1)) // TODO: Correct error code
        }
    }

    const fn internal_new() -> Self {
        Self(false, PhantomData)
    }

    pub fn endpoint_updated(&self, id: chip_EndpointId) {
        unsafe {
            MatterReportingAttributeChangeCallback3(id);
        }
    }

    pub fn attribute_updated(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute_id: chip_AttributeId,
    ) {
        unsafe {
            MatterReportingAttributeChangeCallback1(endpoint_id, cluster_id, attribute_id);
        }
    }

    pub fn schedule(&self, work: extern "C" fn(*mut ()), work_ctx: *mut ()) {
        unsafe {
            Self::platform_mgr().ScheduleWork(Some(core::mem::transmute(work)), work_ctx as _);
        }
    }

    fn platform_mgr() -> &'static mut chip_DeviceLayer_PlatformManager {
        unsafe { chip_DeviceLayer_PlatformMgr().as_mut() }.unwrap()
    }

    // fn configuration_mgr() -> &'static mut chip_DeviceLayer_ConfigurationManager {
    //     unsafe { chip_DeviceLayer_ConfigurationMgr().as_mut() }.unwrap()
    // }

    fn configuration_mgr_impl() -> &'static mut chip_DeviceLayer_ConfigurationManagerImpl {
        unsafe { chip_DeviceLayer_ConfigurationManagerImpl_GetDefaultInstance().as_mut() }.unwrap()
    }

    fn server_init_params() -> &'static mut chip_CommonCaseDeviceServerInitParams {
        unsafe { glue_CommonCaseDeviceServerInitParams().as_mut() }.unwrap()
    }

    fn server() -> &'static mut chip_Server {
        unsafe { chip_Server_GetInstance().as_mut() }.unwrap()
    }
}

impl Drop for ChipContext {
    fn drop(&mut self) {
        if self.0 {
            CTX_TAKEN.store(true, Ordering::SeqCst);
        }
    }
}

pub struct ChipConfiguration<'a> {
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub comissionable_data: Option<&'a dyn cb::ComissionableDataProviderCallback>,
}

impl<'a> ChipConfiguration<'a> {
    pub const fn new() -> Self {
        Self {
            vendor_id: None,
            product_id: None,
            comissionable_data: None,
        }
    }
}

impl<'a> Default for ChipConfiguration<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Chip<'a>(&'a ChipContext, PhantomData<&'a ()>);

impl<'a> Chip<'a> {
    pub fn new(
        context: &'a ChipContext,
        callback: &'a dyn cb::EmberCallback,
        conf: &ChipConfiguration<'a>,
    ) -> Result<Self, ChipError> {
        #[cfg(feature = "log")]
        log::info!(
            "CHIP Build Settings:
                BLE:      {}
                WIFI:     {}
                WPA:      {}
                Thread:   {}
                IPv4:     {}
                TCP:      {}
                EP-COUNT: {}
                DEBUG:    {}\n",
            CONFIG_NETWORK_LAYER_BLE != 0,
            CHIP_DEVICE_CONFIG_ENABLE_WIFI != 0,
            CHIP_DEVICE_CONFIG_ENABLE_WPA != 0,
            CHIP_ENABLE_OPENTHREAD != 0,
            INET_CONFIG_ENABLE_IPV4 != 0,
            INET_CONFIG_ENABLE_TCP_ENDPOINT != 0,
            CHIP_DEVICE_CONFIG_DYNAMIC_ENDPOINT_COUNT,
            CONFIG_IS_DEBUG != 0
        );

        chip!(unsafe { chip_Platform_MemoryInit(core::ptr::null_mut(), 0) })?;
        chip!(unsafe { ChipContext::platform_mgr().InitChipStack() })?;

        unsafe {
            glue_Initialize();
        }

        // TODO: Make conditional
        unsafe {
            chip_Credentials_SetDeviceAttestationCredentialsProvider(
                chip_Credentials_Examples_GetExampleDACProvider(),
            );
        }

        if let Some(vendor_id) = conf.vendor_id {
            chip!(unsafe { ChipContext::configuration_mgr_impl().StoreVendorId(vendor_id) })?;
        }

        if let Some(product_id) = conf.product_id {
            chip!(unsafe { ChipContext::configuration_mgr_impl().StoreProductId(product_id) })?;
        }

        unsafe {
            cb::EMBER = Some(core::mem::transmute(callback));
        }

        if let Some(comissionable_data) = conf.comissionable_data {
            unsafe {
                cb::COMISSIONABLE_DATA_PROVIDER = Some(core::mem::transmute(comissionable_data));
            }
        }

        let init_params = ChipContext::server_init_params();

        chip!(unsafe {
            chip_CommonCaseDeviceServerInitParams_InitializeStaticResourcesBeforeServerInit(
                init_params as *mut _ as *mut _,
            )
        })?;

        chip!(unsafe { ChipContext::server().Init(init_params as *const _ as *const _) })?;

        StaticEndpoint::<0>::initialize()?;

        // TODO
        //ChipContext::configuration_mgr().LogDeviceConfig();

        // TODO
        unsafe {
            PrintOnboardingCodes(chip_RendezvousInformationFlags {
                mValue: chip_RendezvousInformationFlag_kOnNetwork,
                _phantom_0: core::marker::PhantomData,
            });
        }

        Ok(Self(context, PhantomData))
    }

    pub fn context(&self) -> &ChipContext {
        &self.0
    }

    pub fn run(&mut self) {
        unsafe {
            ChipContext::platform_mgr().RunEventLoop();
        }
    }
}

impl<'a> Drop for Chip<'a> {
    fn drop(&mut self) {
        unsafe {
            cb::LOCK = None;
            cb::EMBER = None;
            cb::ACTIONS_PLUGIN_SERVER_INIT = None;
            cb::COMISSIONABLE_DATA_PROVIDER = None;
        }
    }
}

pub trait EmberCallback {
    fn invoke(
        &self,
        _ctx: &ChipContext,
        _command_obj: *mut chip_app_CommandHandler,
        _command_path: *const chip_app_ConcreteCommandPath,
        _command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool {
        false
    }

    fn read(
        &self,
        ctx: &ChipContext,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &mut [u8],
    ) -> Result<(), EmberAfError>;

    fn write(
        &self,
        ctx: &ChipContext,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &[u8],
    ) -> Result<(), EmberAfError>;
}

impl<E> EmberCallback for &E
where
    E: EmberCallback,
{
    fn invoke(
        &self,
        ctx: &ChipContext,
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool {
        (*self).invoke(ctx, command_obj, command_path, command_data)
    }

    fn read(
        &self,
        ctx: &ChipContext,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &mut [u8],
    ) -> Result<(), EmberAfError> {
        (*self).read(ctx, endpoint_id, cluster_id, attribute, buffer)
    }

    fn write(
        &self,
        ctx: &ChipContext,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &[u8],
    ) -> Result<(), EmberAfError> {
        (*self).write(ctx, endpoint_id, cluster_id, attribute, buffer)
    }
}

impl<E> EmberCallback for &mut E
where
    E: EmberCallback,
{
    fn invoke(
        &self,
        ctx: &ChipContext,
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool {
        (**self).invoke(ctx, command_obj, command_path, command_data)
    }

    fn read(
        &self,
        ctx: &ChipContext,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &mut [u8],
    ) -> Result<(), EmberAfError> {
        (**self).read(ctx, endpoint_id, cluster_id, attribute, buffer)
    }

    fn write(
        &self,
        ctx: &ChipContext,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &[u8],
    ) -> Result<(), EmberAfError> {
        (**self).write(ctx, endpoint_id, cluster_id, attribute, buffer)
    }
}

impl<E> cb::EmberCallback for E
where
    E: EmberCallback,
{
    fn cluster_instant_action(
        &self,
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool {
        EmberCallback::invoke(
            &self,
            &ChipContext::internal_new(),
            command_obj,
            command_path,
            command_data,
        )
    }

    fn external_attribute_read(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute_meta_data: *const EmberAfAttributeMetadata,
        buffer: *mut u8,
        max_read_length: u16,
    ) -> EmberAfStatus {
        let attribute = unsafe { (attribute_meta_data as *const Attribute).as_ref() }.unwrap();

        EmberAfError::to_raw(EmberCallback::read(
            self,
            &ChipContext::internal_new(),
            endpoint_id,
            cluster_id,
            attribute,
            unsafe { slice::from_raw_parts_mut(buffer, max_read_length as _) },
        ))
    }

    fn external_attribute_write(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute_meta_data: *const EmberAfAttributeMetadata,
        buffer: *const u8,
    ) -> EmberAfStatus {
        let attribute = unsafe { (attribute_meta_data as *const Attribute).as_ref() }.unwrap();

        EmberAfError::to_raw(EmberCallback::write(
            self,
            &ChipContext::internal_new(),
            endpoint_id,
            cluster_id,
            attribute,
            unsafe { slice::from_raw_parts(buffer, attribute.size()) },
        ))
    }
}

pub struct ComissionableData<'a> {
    pub setup_discriminator: u16,
    pub setup_passcode: u32,
    pub spake2p_iteration_count: u32,
    pub spake2p_salt: &'a [u8],
    pub spake2p_verifier: &'a [u8],
}

impl<'a> cb::ComissionableDataProviderCallback for ComissionableData<'a> {
    fn get_setup_discriminator(&self, setup_discriminator: *mut u16) -> CHIP_ERROR {
        *unsafe { setup_discriminator.as_mut() }.unwrap() = self.setup_discriminator;
        ChipError::from_code(0).error()
    }

    fn get_setup_passcode(&self, setup_passcode: *mut u32) -> CHIP_ERROR {
        *unsafe { setup_passcode.as_mut() }.unwrap() = self.setup_passcode;
        ChipError::from_code(0).error()
    }

    fn get_spake2p_iteration_count(&self, iteration_count: *mut u32) -> CHIP_ERROR {
        *unsafe { iteration_count.as_mut() }.unwrap() = self.spake2p_iteration_count;
        ChipError::from_code(0).error()
    }

    fn get_spake2p_salt(&self, salt_buf: *mut chip_MutableByteSpan) -> CHIP_ERROR {
        let salt_buf = unsafe { salt_buf.as_mut() }.unwrap();
        let salt_data_buf =
            unsafe { core::slice::from_raw_parts_mut(salt_buf.mDataBuf, self.spake2p_salt.len()) };

        salt_data_buf.copy_from_slice(self.spake2p_salt);
        salt_buf.mDataLen = self.spake2p_salt.len();

        ChipError::from_code(0).error()
    }

    fn get_spake2p_verifier(
        &self,
        verifier_buf: *mut chip_MutableByteSpan,
        out_verifier_len: *mut usize,
    ) -> CHIP_ERROR {
        let verifier_buf = unsafe { verifier_buf.as_mut() }.unwrap();
        let verifier_data_buf = unsafe {
            core::slice::from_raw_parts_mut(verifier_buf.mDataBuf, self.spake2p_verifier.len())
        };

        verifier_data_buf.copy_from_slice(self.spake2p_verifier);
        verifier_buf.mDataLen = self.spake2p_verifier.len();

        *unsafe { out_verifier_len.as_mut() }.unwrap() = self.spake2p_verifier.len() as _;

        ChipError::from_code(0).error()
    }
}

pub static TEST_COMISSIONABLE_DATA: ComissionableData<'static> = ComissionableData {
    setup_discriminator: 3840,
    setup_passcode: 20202021,
    spake2p_iteration_count: 1000,
    spake2p_salt: b"SPAKE2P Key Salt",
    spake2p_verifier: &[
        0xB9, 0x61, 0x70, 0xAA, 0xE8, 0x03, 0x34, 0x68, 0x84, 0x72, 0x4F, 0xE9, 0xA3, 0xB2, 0x87,
        0xC3, 0x03, 0x30, 0xC2, 0xA6, 0x60, 0x37, 0x5D, 0x17, 0xBB, 0x20, 0x5A, 0x8C, 0xF1, 0xAE,
        0xCB, 0x35, 0x04, 0x57, 0xF8, 0xAB, 0x79, 0xEE, 0x25, 0x3A, 0xB6, 0xA8, 0xE4, 0x6B, 0xB0,
        0x9E, 0x54, 0x3A, 0xE4, 0x22, 0x73, 0x6D, 0xE5, 0x01, 0xE3, 0xDB, 0x37, 0xD4, 0x41, 0xFE,
        0x34, 0x49, 0x20, 0xD0, 0x95, 0x48, 0xE4, 0xC1, 0x82, 0x40, 0x63, 0x0C, 0x4F, 0xF4, 0x91,
        0x3C, 0x53, 0x51, 0x38, 0x39, 0xB7, 0xC0, 0x7F, 0xCC, 0x06, 0x27, 0xA1, 0xB8, 0x57, 0x3A,
        0x14, 0x9F, 0xCD, 0x1F, 0xA4, 0x66, 0xCF,
    ],
};

pub fn lock<F: FnOnce(&ChipContext) -> R, R>(f: F) -> R {
    cb::lock(|| f(&ChipContext::internal_new()))
}

pub const ENDPOINT_ID_RANGE_START: chip_EndpointId = FIXED_ENDPOINT_COUNT as _;
pub const ENDPOINT_COUNT: usize = CHIP_DEVICE_CONFIG_DYNAMIC_ENDPOINT_COUNT as _;

pub const ROOT_NODE: StaticEndpoint<0> = StaticEndpoint;
pub const BRIDGE_NODE: StaticEndpoint<1> = StaticEndpoint;

const TEMPLATE_NODE: StaticEndpoint<2> = StaticEndpoint;

pub struct StaticEndpoint<const ID: chip_EndpointId>;

impl<const ID: chip_EndpointId> StaticEndpoint<ID> {
    pub const fn id(&self) -> chip_EndpointId {
        ID
    }

    pub fn initialize() -> Result<(), ChipError> {
        static ROOT_DEVICE_TYPES: &[DeviceType] = &[DeviceType::of(0x0016)]; // taken from chip-devices.xml
        ROOT_NODE.initialize_ep(ROOT_DEVICE_TYPES)?;

        //
        // A bridge has root node device type on EP0 and aggregate node device type (bridge) at EP1
        //
        static BRIDGE_NODE_DEVICE_TYPES: &[DeviceType] = &[DeviceType::of(0x000e)]; // taken from chip-devices.xml
        BRIDGE_NODE.initialize_ep(BRIDGE_NODE_DEVICE_TYPES)?;

        // Disable last fixed endpoint, which is used as a placeholder for all of the
        // supported clusters so that ZAP will generate the requisite code.
        TEMPLATE_NODE.enable(false);

        // Disable the bridge EP; users can re-enable
        BRIDGE_NODE.enable(false);

        Ok(())
    }

    fn initialize_ep(&self, device_types: &'static [DeviceType]) -> Result<(), ChipError> {
        lock(|_| {
            chip!(unsafe {
                emberAfSetDeviceTypeList(
                    self.id(),
                    chip_Span {
                        mDataBuf: device_types.as_ptr() as *mut _,
                        mDataLen: device_types.len(),
                        _phantom_0: core::marker::PhantomData,
                    },
                )
            })?;

            Ok(())
        })
    }

    pub fn enable(&self, enable: bool) {
        lock(|_| unsafe {
            emberAfEndpointEnableDisable(self.id(), enable);
        })
    }
}

pub struct EndpointRegistration<'r>(chip_EndpointId, PhantomData<&'r ()>);

impl<'r> EndpointRegistration<'r> {
    pub fn new<const PARENT_ID: chip_EndpointId>(
        id: chip_EndpointId,
        device_types: DeviceTypes<'r>,
        endpoint_type: &'r EndpointType,
        data_versions: &'r mut [chip_DataVersion],
        parent: StaticEndpoint<PARENT_ID>,
    ) -> Result<Self, EmberAfError> {
        lock(|_| {
            if EndpointRegistration::find_index(id).is_some() {
                Err(EmberAfError::from(
                    EmberAfStatus_EMBER_ZCL_STATUS_DUPLICATE_EXISTS,
                ))
            } else if let Some(index) = EndpointRegistration::find_index(chip_kInvalidEndpointId) {
                #[cfg(feature = "log")]
                log::info!("Registering EP {id} at index {index}");

                ember!(unsafe {
                    emberAfSetDynamicEndpoint(
                        index - FIXED_ENDPOINT_COUNT as u16,
                        id,
                        endpoint_type as *const _ as *const _,
                        &chip_Span {
                            mDataBuf: data_versions.as_ptr() as *mut _,
                            mDataLen: data_versions.len(),
                            _phantom_0: core::marker::PhantomData,
                        },
                        chip_Span {
                            mDataBuf: device_types.as_ptr() as *mut _,
                            mDataLen: device_types.len(),
                            _phantom_0: core::marker::PhantomData,
                        },
                        parent.borrow().id(),
                    )
                })?;

                Ok(EndpointRegistration(id, PhantomData))
            } else {
                Err(EmberAfError::from(
                    EmberAfStatus_EMBER_ZCL_STATUS_RESOURCE_EXHAUSTED,
                ))
            }
        })
    }

    pub fn enable(&self, _ctx: &ChipContext, enable: bool) {
        #[cfg(feature = "log")]
        log::info!("Setting enabled state for EP {} to {}", self.id(), enable);

        lock(|_| unsafe {
            emberAfEndpointEnableDisable(
                emberAfEndpointFromIndex(self.index().unwrap() as _),
                enable,
            );
        });
    }

    pub const fn id(&self) -> chip_EndpointId {
        self.0
    }

    fn index(&self) -> Option<u16> {
        Self::find_index(self.0)
    }

    fn find_index(id: chip_EndpointId) -> Option<u16> {
        lock(|_| {
            for index in
                0..(FIXED_ENDPOINT_COUNT + CHIP_DEVICE_CONFIG_DYNAMIC_ENDPOINT_COUNT) as u16
            {
                if unsafe { emberAfEndpointFromIndex(index) } == id {
                    return Some(index as _);
                }
            }

            None
        })
    }
}

impl<'r> Drop for EndpointRegistration<'r> {
    fn drop(&mut self) {
        lock(|_| {
            let index = self.index().unwrap();

            #[cfg(feature = "log")]
            log::info!("Unegistering EP {} from index {index}", self.id());

            unsafe {
                emberAfClearDynamicEndpoint(index - FIXED_ENDPOINT_COUNT as u16);
            }
        });
    }
}

#[repr(transparent)]
pub struct EndpointType<'a, 'c>(EmberAfEndpointType, PhantomData<&'a [&'c ()]>);

impl<'a, 'c> EndpointType<'a, 'c> {
    pub const fn new(clusters: &'a [Cluster<'c>]) -> Self {
        Self(
            EmberAfEndpointType {
                cluster: clusters.as_ptr() as _,
                clusterCount: clusters.len() as _,
                endpointSize: 0,
            },
            PhantomData,
        )
    }

    pub fn clusters(&self) -> ClusterIterator {
        ClusterIterator { ep: self, index: 0 }
    }
}

unsafe impl Send for EndpointType<'static, 'static> {}
unsafe impl<'a, 'c> Sync for EndpointType<'a, 'c> {}

pub struct ClusterIterator<'i, 'a, 'c> {
    ep: &'i EndpointType<'a, 'c>,
    index: usize,
}

impl<'i, 'a, 'c> Iterator for ClusterIterator<'i, 'a, 'c> {
    type Item = &'i Cluster<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.ep.0.clusterCount as _ {
            let clusters =
                unsafe { slice::from_raw_parts(self.ep.0.cluster, self.ep.0.clusterCount as _) };

            let cluster = &clusters[self.index];

            self.index += 1;

            Some(unsafe { (cluster as *const _ as *const Cluster).as_ref() }.unwrap())
        } else {
            None
        }
    }
}

#[repr(transparent)]
pub struct DeviceType(EmberAfDeviceType);

impl DeviceType {
    pub const fn id(&self) -> EmberAfDeviceType {
        self.0
    }

    pub const fn of(id: u16) -> Self {
        Self::new(id, 1)
    }

    pub const fn new(id: u16, version: u8) -> Self {
        Self(EmberAfDeviceType {
            deviceId: id,
            deviceVersion: version,
        })
    }
}

pub type DeviceTypes<'a> = &'a [DeviceType];

#[repr(transparent)]
pub struct Cluster<'a>(EmberAfCluster, PhantomData<&'a ()>);

impl<'a> Cluster<'a> {
    pub const fn new(
        id: chip_ClusterId,
        attributes: &'a [Attribute],
        accepted_commands: &'a [Command],
        generated_commands: &'a [Command],
    ) -> Self {
        Self(
            EmberAfCluster {
                clusterId: id,
                attributes: attributes.as_ptr() as _,
                attributeCount: attributes.len() as _,
                clusterSize: 0,
                mask: CLUSTER_MASK_SERVER as _,
                functions: ptr::null(),
                acceptedCommandList: accepted_commands.as_ptr() as _,
                generatedCommandList: generated_commands.as_ptr() as _,
            },
            PhantomData,
        )
    }

    pub const fn raw(&self) -> &EmberAfCluster {
        &self.0
    }

    pub const fn id(&self) -> chip_ClusterId {
        self.0.clusterId
    }

    pub fn attributes(&self) -> AttributeIterator {
        AttributeIterator {
            cluster: self,
            index: 0,
        }
    }

    pub const fn descriptor() -> Cluster<'static> {
        const ATTRIBUTES: Attributes = &[
            Attribute::array(ZCL_DEVICE_LIST_ATTRIBUTE_ID),
            Attribute::array(ZCL_SERVER_LIST_ATTRIBUTE_ID),
            Attribute::array(ZCL_CLIENT_LIST_ATTRIBUTE_ID),
            Attribute::array(ZCL_PARTS_LIST_ATTRIBUTE_ID),
        ];

        Cluster::new(
            ZCL_DESCRIPTOR_CLUSTER_ID,
            ATTRIBUTES,
            EMPTY_COMMANDS,
            EMPTY_COMMANDS,
        )
    }

    pub const fn bridged() -> Cluster<'static> {
        const ATTRIBUTES: Attributes = &[
            Attribute::string(ZCL_NODE_LABEL_ATTRIBUTE_ID),
            Attribute::boolean(ZCL_REACHABLE_ATTRIBUTE_ID),
        ];

        Cluster::new(
            ZCL_BRIDGED_DEVICE_BASIC_CLUSTER_ID,
            ATTRIBUTES,
            EMPTY_COMMANDS,
            EMPTY_COMMANDS,
        )
    }

    pub const fn on_off() -> Cluster<'static> {
        const ATTRIBUTES: Attributes = &[Attribute::boolean(ZCL_ON_OFF_ATTRIBUTE_ID)];

        Cluster::new(
            ZCL_ON_OFF_CLUSTER_ID,
            ATTRIBUTES,
            EMPTY_COMMANDS,
            EMPTY_COMMANDS,
        )
    }

    pub const fn level_control() -> Cluster<'static> {
        const ATTRIBUTES: Attributes = &[
            Attribute::u8(ZCL_CURRENT_LEVEL_ATTRIBUTE_ID),
            Attribute::u8(ZCL_ON_LEVEL_ATTRIBUTE_ID),
            Attribute::b8(ZCL_OPTIONS_ATTRIBUTE_ID),
        ];

        Cluster::new(
            ZCL_LEVEL_CONTROL_CLUSTER_ID,
            ATTRIBUTES,
            EMPTY_COMMANDS,
            EMPTY_COMMANDS,
        )
    }

    pub const fn target_navigator() -> Cluster<'static> {
        const ATTRIBUTES: Attributes = &[
            Attribute::array(ZCL_TARGET_NAVIGATOR_LIST_ATTRIBUTE_ID),
            Attribute::u8(ZCL_TARGET_NAVIGATOR_CURRENT_TARGET_ATTRIBUTE_ID),
        ];
        const ACCEPTED_COMMANDS: Commands =
            &[Command::new(ZCL_NAVIGATE_TARGET_COMMAND_ID), Command::END];
        const GENERATED_COMMANDS: Commands = &[
            Command::new(ZCL_NAVIGATE_TARGET_RESPONSE_COMMAND_ID),
            Command::END,
        ];

        Cluster::new(
            ZCL_TARGET_NAVIGATOR_CLUSTER_ID,
            ATTRIBUTES,
            ACCEPTED_COMMANDS,
            GENERATED_COMMANDS,
        )
    }

    pub const fn media_playback() -> Cluster<'static> {
        const ATTRIBUTES: Attributes = &[Attribute::u8(ZCL_MEDIA_PLAYBACK_STATE_ATTRIBUTE_ID)];
        const ACCEPTED_COMMANDS: Commands = &[
            Command::new(ZCL_PLAY_COMMAND_ID),
            Command::new(ZCL_PAUSE_COMMAND_ID),
            Command::new(ZCL_STOP_COMMAND_ID),
            Command::END,
        ];
        const GENERATED_COMMANDS: Commands =
            &[Command::new(ZCL_PLAYBACK_RESPONSE_COMMAND_ID), Command::END];

        Cluster::new(
            ZCL_TARGET_NAVIGATOR_CLUSTER_ID,
            ATTRIBUTES,
            ACCEPTED_COMMANDS,
            GENERATED_COMMANDS,
        )
    }

    pub const fn keypad_input() -> Cluster<'static> {
        const ATTRIBUTES: Attributes = &[];
        const ACCEPTED_COMMANDS: Commands = &[Command::new(ZCL_SEND_KEY_COMMAND_ID), Command::END];
        const GENERATED_COMMANDS: Commands =
            &[Command::new(ZCL_SEND_KEY_RESPONSE_COMMAND_ID), Command::END];

        Cluster::new(
            ZCL_TARGET_NAVIGATOR_CLUSTER_ID,
            ATTRIBUTES,
            ACCEPTED_COMMANDS,
            GENERATED_COMMANDS,
        )
    }
}

unsafe impl Send for Cluster<'static> {}
unsafe impl<'a> Sync for Cluster<'a> {}

pub type Clusters<'a, 'c> = &'a [Cluster<'c>];

pub struct AttributeIterator<'i, 'a> {
    cluster: &'i Cluster<'a>,
    index: usize,
}

impl<'i, 'a> Iterator for AttributeIterator<'i, 'a> {
    type Item = &'i Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cluster.0.attributeCount as _ {
            let attributes = unsafe {
                slice::from_raw_parts(
                    self.cluster.0.attributes,
                    self.cluster.0.attributeCount as _,
                )
            };

            let attribute = &attributes[self.index];

            self.index += 1;

            Some(unsafe { (attribute as *const _ as *const Attribute).as_ref() }.unwrap())
        } else {
            None
        }
    }
}

#[repr(transparent)]
pub struct Attribute(EmberAfAttributeMetadata);

unsafe impl Send for Attribute {}
unsafe impl Sync for Attribute {}

impl Attribute {
    pub const fn new(
        id: chip_AttributeId,
        r#type: EmberAfAttributeType,
        size: u16,
        mask: EmberAfAttributeMask,
        //default_value: EmberAfDefaultOrMinMaxAttributeValue,
    ) -> Self {
        Self(EmberAfAttributeMetadata {
            attributeId: id,
            attributeType: r#type,
            size,
            mask: mask | ATTRIBUTE_MASK_EXTERNAL_STORAGE as u8,
            defaultValue: EmberAfDefaultOrMinMaxAttributeValue { defaultValue: 0 },
        })
    }

    pub const fn raw(&self) -> &EmberAfAttributeMetadata {
        &self.0
    }

    pub const fn id(&self) -> chip_AttributeId {
        self.0.attributeId
    }

    pub const fn attr_type(&self) -> EmberAfAttributeType {
        self.0.attributeType
    }

    pub const fn size(&self) -> usize {
        self.0.size as _
    }

    pub const fn boolean(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_BOOLEAN_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn b8(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_BITMAP8_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn b16(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_BITMAP16_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn b32(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_BITMAP32_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn b64(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_BITMAP64_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn u8(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_INT8U_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn u16(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_INT16U_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn u32(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_INT32U_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn u64(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_INT64U_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn i8(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_INT8S_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn i16(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_INT16S_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn i32(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_INT32S_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn i64(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_INT64S_ATTRIBUTE_TYPE as _, 1, 0)
    }

    pub const fn string(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_CHAR_STRING_ATTRIBUTE_TYPE as _, 32, 0)
    }

    pub const fn array(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_ARRAY_ATTRIBUTE_TYPE as _, 254, 0)
    }
}

pub type Attributes<'a> = &'a [Attribute];

#[repr(transparent)]
pub struct Command(chip_CommandId);

impl Command {
    pub const END: Command = Command(0);

    pub const fn id(&self) -> chip_CommandId {
        self.0
    }

    pub const fn new(id: chip_CommandId) -> Self {
        Self(id)
    }
}

pub type Commands<'a> = &'a [Command];

const EMPTY_COMMANDS: Commands = &[Command::END];
