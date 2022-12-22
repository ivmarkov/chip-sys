use crate::*;

static mut LOCK: Option<(&'static dyn Fn(), &'static dyn Fn())> = None;
static mut EMBER_AF: Option<&'static dyn EmberAf> = None;
static mut ACTIONS_PLUGIN_SERVER_INIT: Option<&'static dyn Fn()> = None;
static mut COMISSIONABLE_DATA_PROVIDER: Option<&'static dyn ComissionableDataProvider> = None;

pub trait EmberAf {
    /// # Safety
    /// Manipulates unsafe pointers from CHIP FFI
    unsafe fn invoke(
        &self,
        command_obj: *mut chip_app_CommandHandler,
        command_path: *const chip_app_ConcreteCommandPath,
        command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
    ) -> bool;

    /// # Safety
    /// Manipulates unsafe pointers from CHIP FFI
    unsafe fn read(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute_meta_data: *const EmberAfAttributeMetadata,
        buffer: *const u8,
        max_read_length: u16,
    ) -> EmberAfStatus;

    /// # Safety
    /// Manipulates unsafe pointers from CHIP FFI
    unsafe fn write(
        &self,
        endpoint_id: chip_EndpointId,
        cluster_id: chip_ClusterId,
        attribute_meta_data: *const EmberAfAttributeMetadata,
        buffer: *mut u8,
    ) -> EmberAfStatus;
}

pub trait ComissionableDataProvider {
    /// # Safety
    /// Manipulates unsafe pointers from CHIP FFI
    unsafe fn get_setup_discriminator(
        &self,
        setup_discriminator: *mut u16,
    ) -> Result<(), ChipError>;

    /// # Safety
    /// Manipulates unsafe pointers from CHIP FFI
    unsafe fn get_spake2p_iteration_count(
        &self,
        iteration_count: *mut u32,
    ) -> Result<(), ChipError>;

    /// # Safety
    /// Manipulates unsafe pointers from CHIP FFI
    unsafe fn get_spake2p_salt(&self, salt_buf: *mut chip_MutableByteSpan)
        -> Result<(), ChipError>;

    /// # Safety
    /// Manipulates unsafe pointers from CHIP FFI
    unsafe fn get_spake2p_verifier(
        &self,
        verifier_buf: *mut chip_MutableByteSpan,
        out_verifier_len: *mut usize,
    ) -> Result<(), ChipError>;

    /// # Safety
    /// Manipulates unsafe pointers from CHIP FFI
    unsafe fn get_setup_passcode(&self, setup_passcode: *mut u32) -> Result<(), ChipError>;
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
    af: Option<&'static dyn EmberAf>,
    init: Option<&'static dyn Fn()>,
    provider: Option<&'static dyn ComissionableDataProvider>,
) {
    unsafe {
        LOCK = lock;
        EMBER_AF = af;
        ACTIONS_PLUGIN_SERVER_INIT = init;
        COMISSIONABLE_DATA_PROVIDER = provider;
    }
}

pub struct TestComissionableDataProvider;

impl ComissionableDataProvider for TestComissionableDataProvider {
    unsafe fn get_setup_discriminator(
        &self,
        setup_discriminator: *mut u16,
    ) -> Result<(), ChipError> {
        *unsafe { setup_discriminator.as_mut() }.unwrap() = 3840;

        Ok(())
    }

    unsafe fn get_spake2p_iteration_count(
        &self,
        iteration_count: *mut u32,
    ) -> Result<(), ChipError> {
        *unsafe { iteration_count.as_mut() }.unwrap() = 1000;

        Ok(())
    }

    unsafe fn get_spake2p_salt(
        &self,
        salt_buf: *mut chip_MutableByteSpan,
    ) -> Result<(), ChipError> {
        let salt_buf = unsafe { salt_buf.as_mut() }.unwrap();

        static SALT: &[u8] = b"SPAKE2P Key Salt";

        //VerifyOrReturnError(saltBuf.size() >= kSpake2p_Max_PBKDF_Salt_Length, CHIP_ERROR_BUFFER_TOO_SMALL);

        unsafe { core::slice::from_raw_parts_mut(salt_buf.mDataBuf, SALT.len()) }
            .copy_from_slice(SALT);

        salt_buf.mDataLen = SALT.len();

        Ok(())
    }

    unsafe fn get_spake2p_verifier(
        &self,
        verifier_buf: *mut chip_MutableByteSpan,
        out_verifier_len: *mut usize,
    ) -> Result<(), ChipError> {
        static VERIFIER: &[u8] = &[
            0xB9, 0x61, 0x70, 0xAA, 0xE8, 0x03, 0x34, 0x68, 0x84, 0x72, 0x4F, 0xE9, 0xA3, 0xB2,
            0x87, 0xC3, 0x03, 0x30, 0xC2, 0xA6, 0x60, 0x37, 0x5D, 0x17, 0xBB, 0x20, 0x5A, 0x8C,
            0xF1, 0xAE, 0xCB, 0x35, 0x04, 0x57, 0xF8, 0xAB, 0x79, 0xEE, 0x25, 0x3A, 0xB6, 0xA8,
            0xE4, 0x6B, 0xB0, 0x9E, 0x54, 0x3A, 0xE4, 0x22, 0x73, 0x6D, 0xE5, 0x01, 0xE3, 0xDB,
            0x37, 0xD4, 0x41, 0xFE, 0x34, 0x49, 0x20, 0xD0, 0x95, 0x48, 0xE4, 0xC1, 0x82, 0x40,
            0x63, 0x0C, 0x4F, 0xF4, 0x91, 0x3C, 0x53, 0x51, 0x38, 0x39, 0xB7, 0xC0, 0x7F, 0xCC,
            0x06, 0x27, 0xA1, 0xB8, 0x57, 0x3A, 0x14, 0x9F, 0xCD, 0x1F, 0xA4, 0x66, 0xCF,
        ];
        //static VERIFIER: &'static [u8] = b"uWFwqugDNGiEck/po7KHwwMwwqZgN10XuyBajPGuyzUEV/iree4lOrao5GuwnlQ65CJzbeUB49s31EH+NEkg0JVI5MGCQGMMT/SRPFNRODm3wH/MBiehuFc6FJ/NH6Rmzw==";

        let verifier_buf = unsafe { verifier_buf.as_mut() }.unwrap();

        if verifier_buf.mDataLen < VERIFIER.len() {
            panic!();
        }

        unsafe { core::slice::from_raw_parts_mut(verifier_buf.mDataBuf, VERIFIER.len()) }
            .copy_from_slice(VERIFIER);

        verifier_buf.mDataLen = VERIFIER.len();

        *unsafe { out_verifier_len.as_mut() }.unwrap() = VERIFIER.len();

        Ok(())
    }

    unsafe fn get_setup_passcode(&self, setup_passcode: *mut u32) -> Result<(), ChipError> {
        *unsafe { setup_passcode.as_mut() }.unwrap() = 20202021;

        Ok(())
    }
}

#[no_mangle]
extern "C" fn gluecb_emberAfActionsClusterInstantActionCallback(
    command_obj: *mut chip_app_CommandHandler,
    command_path: *const chip_app_ConcreteCommandPath,
    command_data: *const chip_app_Clusters_Actions_Commands_InstantAction_DecodableType,
) -> bool {
    if let Some(cb) = unsafe { &EMBER_AF } {
        unsafe { cb.invoke(command_obj, command_path, command_data) }
    } else {
        true
    }
}

#[no_mangle]
extern "C" fn gluecb_emberAfExternalAttributeReadCallback(
    endpoint_id: chip_EndpointId,
    cluster_id: chip_ClusterId,
    attribute_meta_data: *const EmberAfAttributeMetadata,
    buffer: *const u8,
    max_read_length: u16,
) -> EmberAfStatus {
    if let Some(cb) = unsafe { &EMBER_AF } {
        unsafe {
            cb.read(
                endpoint_id,
                cluster_id,
                attribute_meta_data,
                buffer,
                max_read_length,
            )
        }
    } else {
        EmberAfStatus_EMBER_ZCL_STATUS_FAILURE
    }
}

#[no_mangle]
extern "C" fn gluecb_emberAfExternalAttributeWriteCallback(
    endpoint_id: chip_EndpointId,
    cluster_id: chip_ClusterId,
    attribute_meta_data: *const EmberAfAttributeMetadata,
    buffer: *mut u8,
) -> EmberAfStatus {
    if let Some(cb) = unsafe { &EMBER_AF } {
        unsafe { cb.write(endpoint_id, cluster_id, attribute_meta_data, buffer) }
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
    let res = if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        unsafe { cb.get_setup_discriminator(setup_discriminator) }
    } else {
        ChipError::convert_code(0x2d)
    };

    ChipError::to_raw(res)
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pIterationCount(
    iteration_count: *mut u32,
) -> CHIP_ERROR {
    let res = if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        unsafe { cb.get_spake2p_iteration_count(iteration_count) }
    } else {
        ChipError::convert_code(0x2d)
    };

    ChipError::to_raw(res)
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pSalt(
    salt_buf: *mut chip_MutableByteSpan,
) -> CHIP_ERROR {
    let res = if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        unsafe { cb.get_spake2p_salt(salt_buf) }
    } else {
        ChipError::convert_code(0x2d)
    };

    ChipError::to_raw(res)
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSpake2pVerifier(
    verifier_buf: *mut chip_MutableByteSpan,
    out_verifier_len: *mut usize,
) -> CHIP_ERROR {
    let res = if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        unsafe { cb.get_spake2p_verifier(verifier_buf, out_verifier_len) }
    } else {
        ChipError::convert_code(0x2d)
    };

    ChipError::to_raw(res)
}

#[no_mangle]
extern "C" fn gluecb_CommissionableDataProvider_GetSetupPasscode(
    setup_passcode: *mut u32,
) -> CHIP_ERROR {
    let res = if let Some(cb) = unsafe { &COMISSIONABLE_DATA_PROVIDER } {
        unsafe { cb.get_setup_passcode(setup_passcode) }
    } else {
        ChipError::convert_code(0x2d)
    };

    ChipError::to_raw(res)
}
