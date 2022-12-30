use crate::*;

static mut LOCK: Option<(&'static dyn Fn(), &'static dyn Fn())> = None;
static mut EMBER: Option<&'static dyn EmberCallback> = None;
static mut ACTIONS_PLUGIN_SERVER_INIT: Option<&'static dyn Fn()> = None;
static mut COMISSIONABLE_DATA_PROVIDER: Option<&'static dyn ComissionableDataProviderCallback> =
    None;

pub trait EmberCallback {
    fn cluster_instant_action(
        &self,
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool;

    fn external_attribute_read(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute: *const EmberAfAttributeMetadata,
        buffer: *mut u8,
        max_read_length: u16,
    ) -> EmberAfStatus;

    fn external_attribute_write(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute_meta_data: *const EmberAfAttributeMetadata,
        buffer: *const u8,
    ) -> EmberAfStatus;
}

pub trait ComissionableDataProviderCallback {
    fn get_setup_discriminator(&self, setup_discriminator: *mut u16) -> CHIP_ERROR;

    fn get_setup_passcode(&self, setup_passcode: *mut u32) -> CHIP_ERROR;

    fn get_spake2p_iteration_count(&self, iteration_count: *mut u32) -> CHIP_ERROR;

    fn get_spake2p_salt(&self, salt_buf: *mut chip_MutableByteSpan) -> CHIP_ERROR;

    fn get_spake2p_verifier(
        &self,
        verifier_buf: *mut chip_MutableByteSpan,
        out_verifier_len: *mut usize,
    ) -> CHIP_ERROR;
}

pub fn lock<F: FnOnce() -> R, R>(f: F) -> R {
    if let Some((lock, unlock)) = unsafe { &LOCK } {
        lock();

        let res = f();

        unlock();

        res
    } else {
        f()
    }
}

/// # Safety
///
/// Call at the beginning of the program when only the main thread is alive.
pub unsafe fn initialize(
    lock: Option<(&'static dyn Fn(), &'static dyn Fn())>,
    af: Option<&'static dyn EmberCallback>,
    init: Option<&'static dyn Fn()>,
    provider: Option<&'static dyn ComissionableDataProviderCallback>,
) {
    unsafe {
        LOCK = lock;
        EMBER = af;
        ACTIONS_PLUGIN_SERVER_INIT = init;
        COMISSIONABLE_DATA_PROVIDER = provider;

        glue_Initialize();
    }
}

#[no_mangle]
extern "C" fn gluecb_emberAfActionsClusterInstantActionCallback(
    command_obj: *mut chip_app_CommandHandler,
    command_path: *const chip_app_ConcreteCommandPath,
    command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
) -> bool {
    if let Some(cb) = unsafe { &EMBER } {
        cb.cluster_instant_action(command_obj, command_path, command_data)
    } else {
        true
    }
}

#[no_mangle]
extern "C" fn gluecb_emberAfExternalAttributeReadCallback(
    endpoint_id: chip_EndpointId,
    cluster_id: chip_ClusterId,
    attribute_meta_data: *const EmberAfAttributeMetadata,
    buffer: *mut u8,
    max_read_length: u16,
) -> EmberAfStatus {
    if let Some(cb) = unsafe { &EMBER } {
        cb.external_attribute_read(
            endpoint_id,
            cluster_id,
            attribute_meta_data,
            buffer,
            max_read_length,
        )
    } else {
        EmberAfStatus_EMBER_ZCL_STATUS_FAILURE
    }
}

#[no_mangle]
extern "C" fn gluecb_emberAfExternalAttributeWriteCallback(
    endpoint_id: chip_EndpointId,
    cluster_id: chip_ClusterId,
    attribute_meta_data: *const EmberAfAttributeMetadata,
    buffer: *const u8,
) -> EmberAfStatus {
    if let Some(cb) = unsafe { &EMBER } {
        cb.external_attribute_write(endpoint_id, cluster_id, attribute_meta_data, buffer)
    } else {
        EmberAfStatus_EMBER_ZCL_STATUS_FAILURE
    }
}

#[no_mangle]
extern "C" fn gluecb_MatterActionsPluginServerInitCallback() {
    if let Some(init) = unsafe { &ACTIONS_PLUGIN_SERVER_INIT } {
        init();
    }
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSetupDiscriminator(
    setup_discriminator: *mut u16,
) -> CHIP_ERROR {
    if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        cb.get_setup_discriminator(setup_discriminator)
    } else {
        ChipError::from_code(0x2d).error()
    }
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pIterationCount(
    iteration_count: *mut u32,
) -> CHIP_ERROR {
    if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        cb.get_spake2p_iteration_count(iteration_count)
    } else {
        ChipError::from_code(0x2d).error()
    }
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pSalt(
    salt_buf: *mut chip_MutableByteSpan,
) -> CHIP_ERROR {
    if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        cb.get_spake2p_salt(salt_buf)
    } else {
        ChipError::from_code(0x2d).error()
    }
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pVerifier(
    verifier_buf: *mut chip_MutableByteSpan,
    out_verifier_len: *mut usize,
) -> CHIP_ERROR {
    if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        cb.get_spake2p_verifier(verifier_buf, out_verifier_len)
    } else {
        ChipError::from_code(0x2d).error()
    }
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSetupPasscode(
    setup_passcode: *mut u32,
) -> CHIP_ERROR {
    if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        cb.get_setup_passcode(setup_passcode)
    } else {
        ChipError::from_code(0x2d).error()
    }
}
