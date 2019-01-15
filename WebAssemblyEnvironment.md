# WebAssembly Environment

This document describes ASL's WebAssembly environment.

## Exports

- `fn configure()`

## Imports

- `fn set_process_name(name_ptr: *const u8, name_len: u32)`
- `fn push_pointer_path(module_ptr: *const u8, module_len: u32, pointer_type: PointerType) -> u32`
- `fn push_offset(pointer_path_id: u32, offset: i64)`
- `fn get_u8(pointer_path_id: u32, current: bool) -> u8`

## Types

### PointerType

i32 with the following values:

| Type   | Value |
| ------ | ----- |
| u8     | 0     |
| u16    | 1     |
| u32    | 2     |
| u64    | 3     |
| i8     | 4     |
| i16    | 5     |
| i32    | 6     |
| i64    | 7     |
| f32    | 8     |
| f64    | 9     |
| String | 10    |
