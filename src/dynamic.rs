// TODO: Probably belongs to `chip-rs` or suchlike separate crate

use core::borrow::Borrow;
use core::fmt::Display;
use core::marker::PhantomData;
use core::ptr;

use crate::callbacks::lock;
use crate::*;

#[cfg(feature = "alloc")]
extern crate alloc;

pub const ENDPOINT_ID_RANGE_START: chip_EndpointId = FIXED_ENDPOINT_COUNT as _;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RegistrationError {
    AlreadyRegistered,
    Overflow,
}

impl Display for RegistrationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AlreadyRegistered => write!(f, "Already registered"),
            Self::Overflow => write!(f, "No more endpoint slots"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RegistrationError {}

pub struct Endpoint<'a, 'c> {
    id: chip_EndpointId,
    ep: EmberAfEndpointType,
    device_types: &'a [DeviceType],
    data_versions: &'a [DataVersion],
    _marker: PhantomData<&'a [&'c ()]>,
}

pub struct StaticEndpoint(chip_EndpointId);

impl<'a, 'c> Endpoint<'a, 'c> {
    pub const fn new(
        id: chip_EndpointId,
        device_types: &'a [DeviceType],
        data_versions: &'a [DataVersion],
        clusters: &'a [Cluster<'c>],
    ) -> Self {
        Self {
            id,
            ep: EmberAfEndpointType {
                cluster: clusters.as_ptr() as _,
                clusterCount: clusters.len() as _,
                endpointSize: 0,
            },
            device_types,
            data_versions,
            _marker: PhantomData,
        }
    }

    pub const fn id(&self) -> chip_EndpointId {
        self.id
    }

    pub fn register<'r>(
        &'r self,
        parent: &'r StaticEndpoint,
    ) -> Result<Registration<'a, 'c, &'r Self>, RegistrationError> {
        Self::register_generic(self, parent)
    }

    #[cfg(feature = "alloc")]
    pub fn register_refcounted(
        self: alloc::rc::Rc<Self>,
        parent: &'static StaticEndpoint,
    ) -> Result<Registration<'a, 'c, alloc::rc::Rc<Self>>, RegistrationError> {
        Self::register_generic(self, parent)
    }

    pub fn register_generic<'r, S>(
        this: S,
        parent: &'r StaticEndpoint,
    ) -> Result<Registration<'a, 'c, S>, RegistrationError>
    where
        S: Borrow<Self>,
    {
        lock(|| {
            let borrowed_this = this.borrow();

            if Registration::<S>::find_index(borrowed_this.id()).is_some() {
                Err(RegistrationError::AlreadyRegistered)
            } else if let Some(index) = Registration::<S>::find_index(chip_kInvalidEndpointId) {
                unsafe {
                    emberAfSetDynamicEndpoint(
                        index as _,
                        borrowed_this.id(),
                        &borrowed_this.ep,
                        &chip_Span {
                            mDataBuf: borrowed_this.data_versions.as_ptr() as *mut _,
                            mDataLen: borrowed_this.data_versions.len(),
                            _phantom_0: core::marker::PhantomData,
                        },
                        chip_Span {
                            mDataBuf: borrowed_this.device_types.as_ptr() as *mut _,
                            mDataLen: borrowed_this.device_types.len(),
                            _phantom_0: core::marker::PhantomData,
                        },
                        parent.borrow().0,
                    );
                }

                Ok(Registration(this, PhantomData))
            } else {
                Err(RegistrationError::Overflow)
            }
        })
    }
}

unsafe impl Send for Endpoint<'static, 'static> {}
unsafe impl<'a, 'c> Sync for Endpoint<'a, 'c> {}

pub struct Registration<'a, 'c, S>(S, PhantomData<(&'a (), &'c ())>)
where
    S: Borrow<Endpoint<'a, 'c>>,
    'c: 'a;

impl<'a, 'c, S> Registration<'a, 'c, S>
where
    S: Borrow<Endpoint<'a, 'c>>,
    'c: 'a,
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

impl<'a, 'c, S> Drop for Registration<'a, 'c, S>
where
    S: Borrow<Endpoint<'a, 'c>>,
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
pub struct DataVersion(chip_DataVersion);

pub type DataVersions<'a> = &'a [DataVersion];

impl DataVersion {
    pub const fn initial() -> Self {
        Self::new(1)
    }

    pub const fn new(version: chip_DataVersion) -> Self {
        Self(version)
    }
}

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

    pub const fn new(id: chip_CommandId) -> Self {
        Self(id)
    }
}

pub type Commands<'a> = &'a [Command];

const EMPTY_COMMANDS: Commands = &[Command::END];

// TODO
pub static ROOT_NODE_REGISTRATION: StaticEndpoint = StaticEndpoint(0);
pub static AGGREGATE_NODE_REGISTRATION: StaticEndpoint = StaticEndpoint(1);

// TODO
pub fn initialize() {
    // (taken from chip-devices.xml)
    const DEVICE_TYPE_ROOT_NODE: u16 = 0x0016;

    // // (taken from chip-devices.xml)
    // const DEVICE_TYPE_BRIDGED_NODE: u16 = 0x0013;

    // (taken from chip-devices.xml)
    const DEVICE_TYPE_BRIDGE: u16 = 0x000e;

    // Device Version for dynamic endpoints:
    const DEVICE_VERSION_DEFAULT: u8 = 1;

    static ROOT_DEVICE_TYPES: &[EmberAfDeviceType] = &[EmberAfDeviceType {
        deviceId: DEVICE_TYPE_ROOT_NODE,
        deviceVersion: DEVICE_VERSION_DEFAULT,
    }];

    static AGGREGATE_NODE_DEVICE_TYPES: &[EmberAfDeviceType] = &[EmberAfDeviceType {
        deviceId: DEVICE_TYPE_BRIDGE,
        deviceVersion: DEVICE_VERSION_DEFAULT,
    }];

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
            ROOT_NODE_REGISTRATION.0,
            chip_Span {
                mDataBuf: ROOT_DEVICE_TYPES.as_ptr() as *mut _,
                mDataLen: ROOT_DEVICE_TYPES.len(),
                _phantom_0: core::marker::PhantomData,
            },
        );
    }
    unsafe {
        emberAfSetDeviceTypeList(
            AGGREGATE_NODE_REGISTRATION.0,
            chip_Span {
                mDataBuf: AGGREGATE_NODE_DEVICE_TYPES.as_ptr() as *mut _,
                mDataLen: AGGREGATE_NODE_DEVICE_TYPES.len(),
                _phantom_0: core::marker::PhantomData,
            },
        );
    }
}
