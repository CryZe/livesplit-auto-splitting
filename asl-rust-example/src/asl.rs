mod sys {
    use super::{PointerKind, State};

    extern "C" {
        pub fn set_process_name(name_ptr: *const u8, name_len: usize);
        pub fn push_pointer_path(
            module_ptr: *const u8,
            module_len: usize,
            kind: PointerKind,
        ) -> usize;
        pub fn push_offset(pointer_path_id: usize, offset: i64);
        pub fn get_u8(pointer_path_id: usize, current: State) -> u8;
        pub fn get_u16(pointer_path_id: usize, current: State) -> u16;
        pub fn get_u32(pointer_path_id: usize, current: State) -> u32;
        pub fn get_u64(pointer_path_id: usize, current: State) -> u64;
        pub fn get_i8(pointer_path_id: usize, current: State) -> i8;
        pub fn get_i16(pointer_path_id: usize, current: State) -> i16;
        pub fn get_i32(pointer_path_id: usize, current: State) -> i32;
        pub fn get_i64(pointer_path_id: usize, current: State) -> i64;
        pub fn get_f32(pointer_path_id: usize, current: State) -> f32;
        pub fn get_f64(pointer_path_id: usize, current: State) -> f64;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PointerKind {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    I8 = 4,
    I16 = 5,
    I32 = 6,
    I64 = 7,
    F32 = 8,
    F64 = 9,
    String = 10,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    Old,
    Current,
}

pub fn set_process_name(module: &str) {
    unsafe {
        sys::set_process_name(module.as_ptr() as *const u8, module.len());
    }
}

pub fn push_pointer_path(module: &str, offsets: &[i64], kind: PointerKind) {
    unsafe {
        let id = sys::push_pointer_path(module.as_ptr() as *const u8, module.len(), kind);
        for &offset in offsets {
            sys::push_offset(id, offset);
        }
    }
}

pub fn get_u8(pointer_path_id: usize, current: State) -> u8 {
    unsafe { sys::get_u8(pointer_path_id, current) }
}

pub fn get_u16(pointer_path_id: usize, current: State) -> u16 {
    unsafe { sys::get_u16(pointer_path_id, current) }
}

pub fn get_u32(pointer_path_id: usize, current: State) -> u32 {
    unsafe { sys::get_u32(pointer_path_id, current) }
}

pub fn get_u64(pointer_path_id: usize, current: State) -> u64 {
    unsafe { sys::get_u64(pointer_path_id, current) }
}

pub fn get_i8(pointer_path_id: usize, current: State) -> i8 {
    unsafe { sys::get_i8(pointer_path_id, current) }
}

pub fn get_i16(pointer_path_id: usize, current: State) -> i16 {
    unsafe { sys::get_i16(pointer_path_id, current) }
}

pub fn get_i32(pointer_path_id: usize, current: State) -> i32 {
    unsafe { sys::get_i32(pointer_path_id, current) }
}

pub fn get_i64(pointer_path_id: usize, current: State) -> i64 {
    unsafe { sys::get_i64(pointer_path_id, current) }
}

pub fn get_f32(pointer_path_id: usize, current: State) -> f32 {
    unsafe { sys::get_f32(pointer_path_id, current) }
}

pub fn get_f64(pointer_path_id: usize, current: State) -> f64 {
    unsafe { sys::get_f64(pointer_path_id, current) }
}

pub trait ASLState
where
    Self: Sized,
{
    fn get() -> (Self, Self);
}
