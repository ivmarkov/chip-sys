use core::cell::Cell;

use chip_sys::chip::{
    Attribute, Chip, ChipConfiguration, ChipContext, Cluster, Clusters, DeviceType, DeviceTypes,
    EmberCallback, EndpointRegistration, EndpointType, BRIDGE_NODE, ENDPOINT_ID_RANGE_START,
    TEST_COMISSIONABLE_DATA,
};

use chip_sys::{
    chip_ClusterId, chip_EndpointId, ChipError, EmberAfError,
    EmberAfStatus_EMBER_ZCL_STATUS_FAILURE, ZCL_BRIDGED_DEVICE_BASIC_CLUSTER_ID,
    ZCL_ON_OFF_ATTRIBUTE_ID, ZCL_ON_OFF_CLUSTER_ID, ZCL_REACHABLE_ATTRIBUTE_ID,
};

static LIGHT_DEVICE_TYPES: DeviceTypes = &[
    DeviceType::of(0x0100), // taken from lo-devices.xml
    DeviceType::of(0x0013), // taken from chip-devices.xml
];

static LIGHT: EndpointType<'static, 'static> = {
    const CLUSTERS: Clusters = &[Cluster::on_off(), Cluster::descriptor(), Cluster::bridged()];

    EndpointType::new(CLUSTERS)
};

pub struct LightState {
    id: chip_EndpointId,
    on: Cell<bool>,
}

impl EmberCallback for LightState {
    fn read(
        &self,
        _ctx: &ChipContext,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &mut [u8],
    ) -> Result<(), EmberAfError> {
        if endpoint_id == self.id
            && cluster_id == ZCL_BRIDGED_DEVICE_BASIC_CLUSTER_ID
            && attribute.id() == ZCL_REACHABLE_ATTRIBUTE_ID
        {
            buffer[0] = 1;

            Ok(())
        } else if endpoint_id == self.id
            && cluster_id == ZCL_ON_OFF_CLUSTER_ID
            && attribute.id() == ZCL_ON_OFF_ATTRIBUTE_ID
        {
            println!("Getting light state: {}", self.on.get());

            buffer[0] = self.on.get() as _;

            Ok(())
        } else {
            Err(EmberAfError::from(EmberAfStatus_EMBER_ZCL_STATUS_FAILURE))
        }
    }

    fn write(
        &self,
        _ctx: &ChipContext,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: &Attribute,
        buffer: &[u8],
    ) -> Result<(), EmberAfError> {
        if endpoint_id == self.id
            && cluster_id == ZCL_ON_OFF_CLUSTER_ID
            && attribute.id() == ZCL_ON_OFF_ATTRIBUTE_ID
        {
            let on = buffer[0] != 0;

            println!("Setting light state to: {}", on);

            self.on.set(on);

            Ok(())
        } else {
            Err(EmberAfError::from(EmberAfStatus_EMBER_ZCL_STATUS_FAILURE))
        }
    }
}

pub fn main() -> Result<(), ChipError> {
    println!("Starting");

    let ctx = ChipContext::take()?;

    let light = LightState {
        id: ENDPOINT_ID_RANGE_START,
        on: Cell::new(false),
    };

    let mut chip = Chip::new(
        &ctx,
        &light,
        &ChipConfiguration {
            comissionable_data: Some(&TEST_COMISSIONABLE_DATA),
            ..Default::default()
        },
    )?;

    BRIDGE_NODE.enable(true);

    let mut data_versions = [0; 8];

    let _registration = EndpointRegistration::new(
        light.id,
        LIGHT_DEVICE_TYPES,
        &LIGHT,
        &mut data_versions,
        BRIDGE_NODE,
    )
    .unwrap();

    chip.run();

    Ok(())
}
