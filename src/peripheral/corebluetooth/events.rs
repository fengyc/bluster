use objc::{
    msg_send,
    runtime::{Object, Sel, NO, YES},
    sel, sel_impl,
};
use objc_foundation::{INSArray, INSString, NSArray, NSObject, NSString};

use super::{
    constants::POWERED_ON_IVAR,
    ffi::{CBATTError, CBManagerState},
    into_bool::IntoBool,
};
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

// TODO: Implement event stream for all below callback

pub extern "C" fn peripheral_manager_did_update_state(
    delegate: &mut Object,
    _cmd: Sel,
    peripheral: *mut Object,
) {
    log::debug!("peripheral_manager_did_update_state");

    unsafe {
        let state: CBManagerState = msg_send![peripheral, state];
        match state {
            CBManagerState::CBManagerStateUnknown => {
                log::debug!("CBManagerStateUnknown");
            }
            CBManagerState::CBManagerStateResetting => {
                log::debug!("CBManagerStateResetting");
            }
            CBManagerState::CBManagerStateUnsupported => {
                log::debug!("CBManagerStateUnsupported");
            }
            CBManagerState::CBManagerStateUnauthorized => {
                log::debug!("CBManagerStateUnauthorized");
            }
            CBManagerState::CBManagerStatePoweredOff => {
                log::debug!("CBManagerStatePoweredOff");
                delegate.set_ivar::<*mut Object>(POWERED_ON_IVAR, NO as *mut Object);
            }
            CBManagerState::CBManagerStatePoweredOn => {
                log::debug!("CBManagerStatePoweredOn");
                delegate.set_ivar::<*mut Object>(POWERED_ON_IVAR, YES as *mut Object);
                let stat = delegate.get_ivar::<*mut Object>(POWERED_ON_IVAR);
                log::debug!("power status: {}", stat.into_bool());
            }
        };
    }
}

pub extern "C" fn peripheral_manager_did_start_advertising_error(
    _delegate: &mut Object,
    _cmd: Sel,
    _peripheral: *mut Object,
    error: *mut Object,
) {
    log::debug!("peripheral_manager_did_start_advertising_error");
    if error.into_bool() {
        let localized_description: *mut Object = unsafe { msg_send![error, localizedDescription] };
        let string = localized_description as *mut NSString;
        log::debug!("{:?}", unsafe { (*string).as_str() });
    }
}

pub extern "C" fn peripheral_manager_did_add_service_error(
    _delegate: &mut Object,
    _cmd: Sel,
    _peripheral: *mut Object,
    _service: *mut Object,
    error: *mut Object,
) {
    log::debug!("peripheral_manager_did_add_service_error");
    if !error.is_null() {
        let localized_description: *mut Object = unsafe { msg_send![error, localizedDescription] };
        let string = localized_description as *mut NSString;
        log::debug!("{:?}", unsafe { (*string).as_str() });
    }
}

pub extern "C" fn peripheral_manager_did_receive_read_request(
    _delegate: &mut Object,
    _cmd: Sel,
    peripheral: *mut Object,
    request: *mut Object,
) {
    unsafe {
        let _: Result<(), ()> = msg_send![peripheral, respondToRequest:request
                                    withResult:CBATTError::CBATTErrorSuccess];
    }
}

pub extern "C" fn peripheral_manager_did_receive_write_requests(
    _delegate: &mut Object,
    _cmd: Sel,
    peripheral: *mut Object,
    requests: *mut Object,
) {
    unsafe {
        for request in (*(requests as *mut NSArray<NSObject>)).to_vec() {
            let _: Result<(), ()> = msg_send![peripheral, respondToRequest:request
                                        withResult:CBATTError::CBATTErrorSuccess];
        }
    }
}
