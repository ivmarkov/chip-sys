// TODO: Probably belongs to `chip-rs` or suchlike separate crate

use core::borrow::{Borrow, BorrowMut};
use core::marker::PhantomData;
use core::{ptr, slice};

use crate::callbacks::lock;
use crate::*;

#[cfg(feature = "alloc")]
extern crate alloc;

pub trait EmberCallback {
    fn invoke(
        &self,
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool;

    fn read(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &mut [u8],
    ) -> Result<(), EmberAfError>;

    fn write(
        &self,
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
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool {
        (*self).invoke(command_obj, command_path, command_data)
    }

    fn read(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &mut [u8],
    ) -> Result<(), EmberAfError> {
        (*self).read(endpoint_id, cluster_id, attribute, buffer)
    }

    fn write(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &[u8],
    ) -> Result<(), EmberAfError> {
        (*self).write(endpoint_id, cluster_id, attribute, buffer)
    }
}

impl<E> EmberCallback for &mut E
where
    E: EmberCallback,
{
    fn invoke(
        &self,
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool {
        (**self).invoke(command_obj, command_path, command_data)
    }

    fn read(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &mut [u8],
    ) -> Result<(), EmberAfError> {
        (**self).read(endpoint_id, cluster_id, attribute, buffer)
    }

    fn write(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &[u8],
    ) -> Result<(), EmberAfError> {
        (**self).write(endpoint_id, cluster_id, attribute, buffer)
    }
}

impl<E> callbacks::EmberCallback for E
where
    E: EmberCallback,
{
    fn cluster_instant_action(
        &self,
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool {
        EmberCallback::invoke(&self, command_obj, command_path, command_data)
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
            endpoint_id,
            cluster_id,
            attribute,
            unsafe { slice::from_raw_parts(buffer, attribute.size()) },
        ))
    }
}

pub trait ComissionableDataProviderCallback {
    fn setup_discriminator(&self) -> Result<u16, ChipError>;
    fn setup_passcode(&self) -> Result<u32, ChipError>;

    fn spake2p_iteration_count(&self) -> Result<u32, ChipError>;
    fn spake2p_salt(&self) -> Result<&[u8], ChipError>;
    fn spake2p_verifier(&self) -> Result<&[u8], ChipError>;
}

impl<C> ComissionableDataProviderCallback for &C
where
    C: ComissionableDataProviderCallback,
{
    fn setup_discriminator(&self) -> Result<u16, ChipError> {
        (*self).setup_discriminator()
    }

    fn setup_passcode(&self) -> Result<u32, ChipError> {
        (*self).setup_passcode()
    }

    fn spake2p_iteration_count(&self) -> Result<u32, ChipError> {
        (*self).spake2p_iteration_count()
    }

    fn spake2p_salt(&self) -> Result<&[u8], ChipError> {
        (*self).spake2p_salt()
    }

    fn spake2p_verifier(&self) -> Result<&[u8], ChipError> {
        (*self).spake2p_verifier()
    }
}

impl<C> ComissionableDataProviderCallback for &mut C
where
    C: ComissionableDataProviderCallback,
{
    fn setup_discriminator(&self) -> Result<u16, ChipError> {
        (**self).setup_discriminator()
    }

    fn setup_passcode(&self) -> Result<u32, ChipError> {
        (**self).setup_passcode()
    }

    fn spake2p_iteration_count(&self) -> Result<u32, ChipError> {
        (**self).spake2p_iteration_count()
    }

    fn spake2p_salt(&self) -> Result<&[u8], ChipError> {
        (**self).spake2p_salt()
    }

    fn spake2p_verifier(&self) -> Result<&[u8], ChipError> {
        (**self).spake2p_verifier()
    }
}

impl<C> callbacks::ComissionableDataProviderCallback for C
where
    C: ComissionableDataProviderCallback,
{
    fn get_setup_discriminator(&self, setup_discriminator: *mut u16) -> CHIP_ERROR {
        let err = match ComissionableDataProviderCallback::setup_discriminator(&self) {
            Ok(value) => {
                *unsafe { setup_discriminator.as_mut() }.unwrap() = value;
                ChipError::from_code(0)
            }
            Err(err) => err,
        };

        err.error()
    }

    fn get_setup_passcode(&self, setup_passcode: *mut u32) -> CHIP_ERROR {
        let err = match ComissionableDataProviderCallback::setup_passcode(&self) {
            Ok(value) => {
                *unsafe { setup_passcode.as_mut() }.unwrap() = value;
                ChipError::from_code(0)
            }
            Err(err) => err,
        };

        err.error()
    }

    fn get_spake2p_iteration_count(&self, iteration_count: *mut u32) -> CHIP_ERROR {
        let err = match ComissionableDataProviderCallback::spake2p_iteration_count(&self) {
            Ok(value) => {
                *unsafe { iteration_count.as_mut() }.unwrap() = value;
                ChipError::from_code(0)
            }
            Err(err) => err,
        };

        err.error()
    }

    fn get_spake2p_salt(&self, salt_buf: *mut chip_MutableByteSpan) -> CHIP_ERROR {
        //VerifyOrReturnError(saltBuf.size() >= kSpake2p_Max_PBKDF_Salt_Length, CHIP_ERROR_BUFFER_TOO_SMALL);

        let err = match ComissionableDataProviderCallback::spake2p_salt(&self) {
            Ok(value) => {
                let salt_buf = unsafe { salt_buf.as_mut() }.unwrap();
                let salt_data_buf =
                    unsafe { core::slice::from_raw_parts_mut(salt_buf.mDataBuf, value.len()) };

                salt_data_buf.copy_from_slice(value);
                salt_buf.mDataLen = value.len();

                ChipError::from_code(0)
            }
            Err(err) => err,
        };

        err.error()
    }

    fn get_spake2p_verifier(
        &self,
        verifier_buf: *mut chip_MutableByteSpan,
        out_verifier_len: *mut usize,
    ) -> CHIP_ERROR {
        let err = match ComissionableDataProviderCallback::spake2p_salt(&self) {
            Ok(value) => {
                let verifier_buf = unsafe { verifier_buf.as_mut() }.unwrap();
                let verifier_data_buf =
                    unsafe { core::slice::from_raw_parts_mut(verifier_buf.mDataBuf, value.len()) };

                verifier_data_buf.copy_from_slice(value);
                verifier_buf.mDataLen = value.len();

                *unsafe { out_verifier_len.as_mut() }.unwrap() = value.len() as _;

                ChipError::from_code(0)
            }
            Err(err) => err,
        };

        err.error()
    }
}

pub struct TestComissionableDataProvider;

impl ComissionableDataProviderCallback for TestComissionableDataProvider {
    fn setup_discriminator(&self) -> Result<u16, ChipError> {
        Ok(3840)
    }

    fn setup_passcode(&self) -> Result<u32, ChipError> {
        Ok(20202021)
    }

    fn spake2p_iteration_count(&self) -> Result<u32, ChipError> {
        Ok(1000)
    }

    fn spake2p_salt(&self) -> Result<&[u8], ChipError> {
        Ok(b"SPAKE2P Key Salt")
    }

    fn spake2p_verifier(&self) -> Result<&[u8], ChipError> {
        //static VERIFIER: &'static [u8] = b"uWFwqugDNGiEck/po7KHwwMwwqZgN10XuyBajPGuyzUEV/iree4lOrao5GuwnlQ65CJzbeUB49s31EH+NEkg0JVI5MGCQGMMT/SRPFNRODm3wH/MBiehuFc6FJ/NH6Rmzw==";

        Ok(&[
            0xB9, 0x61, 0x70, 0xAA, 0xE8, 0x03, 0x34, 0x68, 0x84, 0x72, 0x4F, 0xE9, 0xA3, 0xB2,
            0x87, 0xC3, 0x03, 0x30, 0xC2, 0xA6, 0x60, 0x37, 0x5D, 0x17, 0xBB, 0x20, 0x5A, 0x8C,
            0xF1, 0xAE, 0xCB, 0x35, 0x04, 0x57, 0xF8, 0xAB, 0x79, 0xEE, 0x25, 0x3A, 0xB6, 0xA8,
            0xE4, 0x6B, 0xB0, 0x9E, 0x54, 0x3A, 0xE4, 0x22, 0x73, 0x6D, 0xE5, 0x01, 0xE3, 0xDB,
            0x37, 0xD4, 0x41, 0xFE, 0x34, 0x49, 0x20, 0xD0, 0x95, 0x48, 0xE4, 0xC1, 0x82, 0x40,
            0x63, 0x0C, 0x4F, 0xF4, 0x91, 0x3C, 0x53, 0x51, 0x38, 0x39, 0xB7, 0xC0, 0x7F, 0xCC,
            0x06, 0x27, 0xA1, 0xB8, 0x57, 0x3A, 0x14, 0x9F, 0xCD, 0x1F, 0xA4, 0x66, 0xCF,
        ])
    }
}

pub const ENDPOINT_ID_RANGE_START: chip_EndpointId = FIXED_ENDPOINT_COUNT as _;

pub struct StaticEndpoint(chip_EndpointId);

impl StaticEndpoint {
    pub const fn id(&self) -> chip_EndpointId {
        self.0
    }

    fn initialize(&self, device_types: &'static [DeviceType]) -> Result<(), ChipError> {
        chip!(unsafe {
            emberAfSetDeviceTypeList(
                self.0,
                chip_Span {
                    mDataBuf: device_types.as_ptr() as *mut _,
                    mDataLen: device_types.len(),
                    _phantom_0: core::marker::PhantomData,
                },
            )
        })?;

        Ok(())
    }
}

pub struct Endpoint<'a, 'c> {
    id: chip_EndpointId,
    ep: EmberAfEndpointType,
    device_types: &'a [DeviceType],
    clusters: &'a [Cluster<'c>],
    _marker: PhantomData<&'a [&'c ()]>,
}

impl<'a, 'c> Endpoint<'a, 'c> {
    pub const fn new(
        id: chip_EndpointId,
        device_types: &'a [DeviceType],
        clusters: &'a [Cluster<'c>],
    ) -> Self {
        Self {
            id,
            ep: EmberAfEndpointType {
                cluster: clusters.as_ptr() as _,
                clusterCount: clusters.len() as _,
                endpointSize: 0,
            },
            clusters,
            device_types,
            _marker: PhantomData,
        }
    }

    pub const fn id(&self) -> chip_EndpointId {
        self.id
    }

    pub const fn clusters(&self) -> &[Cluster<'c>] {
        self.clusters
    }

    pub fn register<'r>(
        &'r self,
        data_versions: &'r mut [chip_DataVersion],
        parent: &'r StaticEndpoint,
    ) -> Result<Registration<'r, 'a, 'c, &'r Self, &'r mut [chip_DataVersion]>, EmberAfError> {
        Self::register_generic(self, data_versions, parent)
    }

    #[cfg(feature = "alloc")]
    pub fn register_refcounted(
        self: alloc::rc::Rc<Self>,
        data_versions: alloc::vec::Vec<chip_DataVersion>,
        parent: &'static StaticEndpoint,
    ) -> Result<
        Registration<'static, 'a, 'c, alloc::rc::Rc<Self>, alloc::vec::Vec<chip_DataVersion>>,
        EmberAfError,
    > {
        Self::register_generic(self, data_versions, parent)
    }

    pub fn register_generic<'r, S, V>(
        this: S,
        mut data_versions: V,
        parent: &'r StaticEndpoint,
    ) -> Result<Registration<'r, 'a, 'c, S, V>, EmberAfError>
    where
        S: Borrow<Self> + 'r,
        V: BorrowMut<[chip_DataVersion]>,
        'a: 'r,
        'c: 'r,
    {
        lock(|| {
            let borrowed_this = this.borrow();

            if let Some(index) = Registration::<S, V>::find_index(chip_kInvalidEndpointId) {
                let borrowed_data_versions = data_versions.borrow_mut();

                ember!(unsafe {
                    emberAfSetDynamicEndpoint(
                        index as _,
                        borrowed_this.id(),
                        &borrowed_this.ep,
                        &chip_Span {
                            mDataBuf: borrowed_data_versions.as_ptr() as *mut _,
                            mDataLen: borrowed_data_versions.len(),
                            _phantom_0: core::marker::PhantomData,
                        },
                        chip_Span {
                            mDataBuf: borrowed_this.device_types.as_ptr() as *mut _,
                            mDataLen: borrowed_this.device_types.len(),
                            _phantom_0: core::marker::PhantomData,
                        },
                        parent.borrow().0,
                    )
                })?;

                Ok(Registration(this, data_versions, PhantomData))
            } else {
                Err(EmberAfError::from(
                    EmberAfStatus_EMBER_ZCL_STATUS_RESOURCE_EXHAUSTED,
                ))
            }
        })
    }
}

unsafe impl Send for Endpoint<'static, 'static> {}
unsafe impl<'a, 'c> Sync for Endpoint<'a, 'c> {}

pub struct Registration<'r, 'a, 'c, S, V>(S, V, PhantomData<(&'r (), &'a (), &'c ())>)
where
    S: Borrow<Endpoint<'a, 'c>> + 'r,
    'c: 'a,
    'a: 'r;

impl<'r, 'a, 'c, S, V> Registration<'r, 'a, 'c, S, V>
where
    S: Borrow<Endpoint<'a, 'c>> + 'r,
{
    pub fn enable(&self, enable: bool) {
        lock(|| unsafe {
            emberAfEndpointEnableDisable(
                emberAfEndpointFromIndex(self.index().unwrap() as _),
                enable,
            );
        });
    }

    fn index(&self) -> Option<usize> {
        Self::find_index(self.0.borrow().id)
    }

    fn find_index(id: chip_EndpointId) -> Option<usize> {
        for index in 0..CHIP_DEVICE_CONFIG_DYNAMIC_ENDPOINT_COUNT as _ {
            if unsafe { emberAfEndpointFromIndex(index as _) } == id {
                return Some(index);
            }
        }

        None
    }
}

impl<'r, 'a, 'c, S, V> Drop for Registration<'r, 'a, 'c, S, V>
where
    S: Borrow<Endpoint<'a, 'c>> + 'r,
{
    fn drop(&mut self) {
        let index = self.index().unwrap();

        lock(|| unsafe {
            emberAfClearDynamicEndpoint(index as _);
        });
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

// #[repr(transparent)]
// #[derive(Copy, Clone, Eq, PartialEq, Default)]
// pub struct DataVersion(chip_DataVersion);

// pub type DataVersions<'a> = &'a [DataVersion];

// impl DataVersion {
//     pub const fn new() -> Self {
//         Self(0)
//     }
// }

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
        const ATTRIBUTES: &[Attribute] = &[
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
        const ATTRIBUTES: &[Attribute] = &[Attribute::boolean(ZCL_ON_OFF_ATTRIBUTE_ID)];

        Cluster::new(
            ZCL_ON_OFF_CLUSTER_ID,
            ATTRIBUTES,
            EMPTY_COMMANDS,
            EMPTY_COMMANDS,
        )
    }
}

unsafe impl Send for Cluster<'static> {}
unsafe impl<'a> Sync for Cluster<'a> {}

pub type Clusters<'a, 'c> = &'a [Cluster<'c>];

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

    pub const fn size(&self) -> usize {
        16 // TODO
    }

    pub const fn boolean(id: chip_AttributeId) -> Self {
        Self::new(id, ZCL_BOOLEAN_ATTRIBUTE_TYPE as _, 1, 0)
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

// TODO
pub static ROOT_NODE: StaticEndpoint = StaticEndpoint(0);
pub static BRIDGE_NODE: StaticEndpoint = StaticEndpoint(1);

// TODO
pub fn initialize() -> Result<(), ChipError> {
    // Disable last fixed endpoint, which is used as a placeholder for all of the
    // supported clusters so that ZAP will generated the requisite code.
    unsafe {
        emberAfEndpointEnableDisable(
            emberAfEndpointFromIndex(emberAfFixedEndpointCount() - 1),
            false,
        );
    }

    //
    // A bridge has root node device type on EP0 and aggregate node device type (bridge) at EP1
    //

    static ROOT_DEVICE_TYPES: &[DeviceType] = &[DeviceType::of(0x0016)]; // taken from chip-devices.xml
    ROOT_NODE.initialize(ROOT_DEVICE_TYPES)?;

    static BRIDGE_NODE_DEVICE_TYPES: &[DeviceType] = &[DeviceType::of(0x000e)]; // taken from chip-devices.xml
    BRIDGE_NODE.initialize(BRIDGE_NODE_DEVICE_TYPES)?;

    Ok(())
}
