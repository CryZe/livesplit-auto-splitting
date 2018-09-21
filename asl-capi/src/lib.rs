extern crate asl_runtime;

use asl_runtime::{Runtime, TimerAction, TimerState};
use std::ffi::CStr;
use std::fs;
use std::os::raw::c_char;

unsafe fn str(s: *const c_char) -> &'static str {
    if s.is_null() {
        ""
    } else {
        CStr::from_ptr(s as _).to_str().unwrap()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ASLRuntime_from_path(path: *const c_char) -> Option<Box<Runtime>> {
    let script = fs::read(str(path)).ok()?;
    Some(Box::new(Runtime::new(&script).ok()?))
}

#[no_mangle]
pub extern "C" fn ASLRuntime_drop(this: Box<Runtime>) {
    drop(this);
}

#[no_mangle]
pub extern "C" fn ASLRuntime_step(this: &mut Runtime) -> i32 {
    match this.step() {
        Err(_) => -1,
        Ok(None) => 0,
        Ok(Some(TimerAction::Start)) => 1,
        Ok(Some(TimerAction::Split)) => 2,
        Ok(Some(TimerAction::Reset)) => 3,
    }
}

#[no_mangle]
pub extern "C" fn ASLRuntime_set_state(this: &mut Runtime, state: TimerState) {
    this.set_state(state)
}
