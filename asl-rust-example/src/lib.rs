#[macro_use]
extern crate asl_derive;

mod asl;

use asl::ASLState;

#[derive(ASLState)]
#[Process = "bgb.exe"]
struct MyState {
    #[Pointer = "bgb.exe, 0x00166EDC, 0x274, 0x362"]
    x: u8,
}

#[no_mangle]
pub extern "C" fn should_start() -> bool {
    let (current, old) = MyState::get();

    current.x >= 5 && old.x < 5
}
