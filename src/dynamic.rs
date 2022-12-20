// TODO: Probably belongs to `chip-rs` or suchlike separate crate

use core::marker::PhantomData;
use core::ptr;

use crate::*;

pub fn lock<F: FnOnce() -> R, R>(f: F) -> R {
    f() // TODO
}

pub struct Endpoint<'a, 'c> {
    id: chip_EndpointId,
    ep: EmberAfEndpointType,
    device_types: &'a [DeviceType],
    data_versions: &'a [DataVersion],
    _marker: PhantomData<&'a [&'c ()]>,
}

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

    pub fn register<'r, 'p>(
        &'r self,
        parent: &'r Registration<'p>,
    ) -> Result<Registration<'r>, ()> {
        lock(|| {
            if Registration::find_index(self.id()).is_some() {
                Err(())
            } else if let Some(index) = Registration::find_index(chip_kInvalidEndpointId) {
                unsafe {
                    emberAfSetDynamicEndpoint(
                        index as _,
                        self.id(),
                        &self.ep,
                        &chip_Span {
                            mDataBuf: self.data_versions.as_ptr() as *mut _,
                            mDataLen: self.data_versions.len(),
                            _phantom_0: core::marker::PhantomData,
                        },
                        chip_Span {
                            mDataBuf: self.device_types.as_ptr() as *mut _,
                            mDataLen: self.device_types.len(),
                            _phantom_0: core::marker::PhantomData,
                        },
                        parent.0,
                    );
                }

                Ok(Registration(self.id(), PhantomData))
            } else {
                Err(())
            }
        })
    }
}

pub struct Registration<'r>(chip_EndpointId, PhantomData<&'r ()>);

impl<'r> Registration<'r> {
    pub fn enable(&self, enable: bool) {
        lock(|| unsafe {
            emberAfEndpointEnableDisable(
                emberAfEndpointFromIndex(self.index().unwrap() as _),
                enable,
            );
        });
    }

    fn index(&self) -> Option<usize> {
        Self::find_index(self.0)
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

impl<'r> Drop for Registration<'r> {
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
                clusterSize: 0, // TODO
                mask: 0,        // TODO
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
pub static ROOT_NODE_REGISTRATION: Registration<'static> = Registration(0, PhantomData);
pub static AGGREGATE_NODE_REGISTRATION: Registration<'static> = Registration(1, PhantomData);

pub const ENDPOINT_ID_RANGE_START: chip_EndpointId = FIXED_ENDPOINT_COUNT as _;

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
