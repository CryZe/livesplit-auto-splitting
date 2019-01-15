use num_traits::FromPrimitive;
use pointer::{PointerType, PointerValue};
use std::{fmt, str};
use wasmi::{
    Error, Externals, FuncInstance, FuncRef, GlobalDescriptor, GlobalRef, HostError,
    ImportResolver, MemoryDescriptor, MemoryRef, RuntimeArgs, RuntimeValue, Signature,
    TableDescriptor, TableRef, Trap, TrapKind, ValueType,
};

const SET_PROCESS_NAME_FUNC_INDEX: usize = 0;
const PUSH_POINTER_PATH_FUNC_INDEX: usize = 1;
const PUSH_OFFSET_FUNC_INDEX: usize = 2;
const GET_U8_FUNC_INDEX: usize = 3;
const GET_U16_FUNC_INDEX: usize = 4;
const GET_U32_FUNC_INDEX: usize = 5;
const GET_U64_FUNC_INDEX: usize = 6;
const GET_I8_FUNC_INDEX: usize = 7;
const GET_I16_FUNC_INDEX: usize = 8;
const GET_I32_FUNC_INDEX: usize = 9;
const GET_I64_FUNC_INDEX: usize = 10;
const GET_F32_FUNC_INDEX: usize = 11;
const GET_F64_FUNC_INDEX: usize = 12;

#[derive(Debug)]
enum EnvironmentError {
    InvalidProcessName,
    InvalidModuleName,
    InvalidPointerPathId,
    InvalidPointerType,
    TypeMismatch,
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EnvironmentError::InvalidProcessName => write!(f, "Invalid process name"),
            EnvironmentError::InvalidModuleName => {
                write!(f, "Invalid module name provided to construct pointer path")
            }
            EnvironmentError::InvalidPointerPathId => write!(f, "Invalid pointer path id provided"),
            EnvironmentError::InvalidPointerType => write!(f, "Invalid pointer type provided"),
            EnvironmentError::TypeMismatch => {
                write!(f, "Attempt to read from a value of the wrong type")
            }
        }
    }
}

impl HostError for EnvironmentError {}

#[derive(Debug)]
pub struct Environment {
    memory: MemoryRef,
    pub process_name: String,
    // TODO Undo pub
    pub pointer_paths: Vec<PointerPath>,
}

#[derive(Debug)]
pub struct PointerPath {
    pub module_name: String,
    pub offsets: Vec<i64>,
    // TODO Undo pub
    pub current: PointerValue,
    pub old: PointerValue,
}

impl Environment {
    pub fn new(memory: MemoryRef) -> Self {
        Self {
            memory,
            process_name: String::new(),
            pointer_paths: Vec::new(),
        }
    }
}

impl Externals for Environment {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            SET_PROCESS_NAME_FUNC_INDEX => {
                let ptr: u32 = args.nth_checked(0)?;
                let ptr = ptr as usize;
                let len: u32 = args.nth_checked(1)?;
                let len = len as usize;

                self.process_name = self
                    .memory
                    .with_direct_access(|m| {
                        Some(str::from_utf8(m.get(ptr..ptr + len)?).ok()?.to_owned())
                    }).ok_or_else(|| {
                        Trap::new(TrapKind::Host(Box::new(
                            EnvironmentError::InvalidProcessName,
                        )))
                    })?;

                Ok(None)
            }
            PUSH_POINTER_PATH_FUNC_INDEX => {
                let ptr: u32 = args.nth_checked(0)?;
                let ptr = ptr as usize;
                let len: u32 = args.nth_checked(1)?;
                let len = len as usize;
                let pointer_type: u8 = args.nth_checked(2)?;
                let pointer_type = PointerType::from_u8(pointer_type).ok_or_else(|| {
                    Trap::new(TrapKind::Host(Box::new(
                        EnvironmentError::InvalidPointerType,
                    )))
                })?;
                let current = match pointer_type {
                    PointerType::U8 => PointerValue::U8(0),
                    PointerType::U16 => PointerValue::U16(0),
                    PointerType::U32 => PointerValue::U32(0),
                    PointerType::U64 => PointerValue::U64(0),
                    PointerType::I8 => PointerValue::I8(0),
                    PointerType::I16 => PointerValue::I16(0),
                    PointerType::I32 => PointerValue::I32(0),
                    PointerType::I64 => PointerValue::I64(0),
                    PointerType::F32 => PointerValue::F32(0.0),
                    PointerType::F64 => PointerValue::F64(0.0),
                    PointerType::String => PointerValue::String(String::new()),
                };

                let module_name = self
                    .memory
                    .with_direct_access(|m| {
                        Some(str::from_utf8(m.get(ptr..ptr + len)?).ok()?.to_owned())
                    }).ok_or_else(|| {
                        Trap::new(TrapKind::Host(Box::new(
                            EnvironmentError::InvalidModuleName,
                        )))
                    })?;

                let id = self.pointer_paths.len();
                self.pointer_paths.push(PointerPath {
                    module_name,
                    offsets: Vec::new(),
                    old: current.clone(),
                    current,
                });

                Ok(Some(RuntimeValue::I32(id as i32)))
            }
            PUSH_OFFSET_FUNC_INDEX => {
                let pointer_path_id: u32 = args.nth_checked(0)?;
                let pointer_path_id = pointer_path_id as usize;
                let offset: i64 = args.nth_checked(1)?;
                let pointer_path =
                    self.pointer_paths.get_mut(pointer_path_id).ok_or_else(|| {
                        Trap::new(TrapKind::Host(Box::new(
                            EnvironmentError::InvalidPointerPathId,
                        )))
                    })?;
                pointer_path.offsets.push(offset);
                Ok(None)
            }
            GET_U8_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                PointerValue::U8(v) => Some(RuntimeValue::I32(*v as i32)),
                _ => None,
            }),
            GET_U16_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                PointerValue::U16(v) => Some(RuntimeValue::I32(*v as i32)),
                _ => None,
            }),
            GET_U32_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                PointerValue::U32(v) => Some(RuntimeValue::I32(*v as i32)),
                _ => None,
            }),
            GET_U64_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                PointerValue::U64(v) => Some(RuntimeValue::I64(*v as i64)),
                _ => None,
            }),
            GET_I8_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                PointerValue::I8(v) => Some(RuntimeValue::I32(*v as i32)),
                _ => None,
            }),
            GET_I16_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                PointerValue::I16(v) => Some(RuntimeValue::I32(*v as i32)),
                _ => None,
            }),
            GET_I32_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                PointerValue::I32(v) => Some(RuntimeValue::I32(*v)),
                _ => None,
            }),
            GET_I64_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                PointerValue::I64(v) => Some(RuntimeValue::I64(*v)),
                _ => None,
            }),
            GET_F32_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                &PointerValue::F32(v) => Some(RuntimeValue::F32(v.into())),
                _ => None,
            }),
            GET_F64_FUNC_INDEX => get_val(args, &self.pointer_paths, |v| match v {
                &PointerValue::F64(v) => Some(RuntimeValue::F64(v.into())),
                _ => None,
            }),
            _ => panic!("Unimplemented function at {}", index),
        }
    }
}

pub struct Imports;

impl ImportResolver for Imports {
    fn resolve_func(
        &self,
        _module_name: &str,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, Error> {
        let instance = match field_name {
            "set_process_name" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                SET_PROCESS_NAME_FUNC_INDEX,
            ),
            "push_pointer_path" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32),
                ),
                PUSH_POINTER_PATH_FUNC_INDEX,
            ),
            "push_offset" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I64][..], None),
                PUSH_OFFSET_FUNC_INDEX,
            ),
            "get_u8" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                GET_U8_FUNC_INDEX,
            ),
            "get_u16" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                GET_U16_FUNC_INDEX,
            ),
            "get_u32" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                GET_U32_FUNC_INDEX,
            ),
            "get_u64" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I64)),
                GET_U64_FUNC_INDEX,
            ),
            "get_i8" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                GET_I8_FUNC_INDEX,
            ),
            "get_i16" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                GET_I16_FUNC_INDEX,
            ),
            "get_i32" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                GET_I32_FUNC_INDEX,
            ),
            "get_i64" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I64)),
                GET_I64_FUNC_INDEX,
            ),
            "get_f32" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::F32)),
                GET_F32_FUNC_INDEX,
            ),
            "get_f64" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::F64)),
                GET_F64_FUNC_INDEX,
            ),
            _ => {
                return Err(Error::Instantiation(format!(
                    "Export {} not found",
                    field_name
                )))
            }
        };
        Ok(instance)
    }

    fn resolve_global(
        &self,
        _module_name: &str,
        _field_name: &str,
        _descriptor: &GlobalDescriptor,
    ) -> Result<GlobalRef, Error> {
        Err(Error::Instantiation("Global not found".to_string()))
    }
    fn resolve_memory(
        &self,
        _module_name: &str,
        _field_name: &str,
        _descriptor: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        Err(Error::Instantiation("Memory not found".to_string()))
    }
    fn resolve_table(
        &self,
        _module_name: &str,
        _field_name: &str,
        _descriptor: &TableDescriptor,
    ) -> Result<TableRef, Error> {
        Err(Error::Instantiation("Table not found".to_string()))
    }
}

fn get_val(
    args: RuntimeArgs,
    pointer_paths: &[PointerPath],
    convert: impl FnOnce(&PointerValue) -> Option<RuntimeValue>,
) -> Result<Option<RuntimeValue>, Trap> {
    let pointer_path_id: u32 = args.nth_checked(0)?;
    let pointer_path_id = pointer_path_id as usize;
    let current: bool = args.nth_checked(1)?;

    let pointer_path = pointer_paths.get(pointer_path_id).ok_or_else(|| {
        Trap::new(TrapKind::Host(Box::new(
            EnvironmentError::InvalidPointerPathId,
        )))
    })?;
    let value = if current {
        &pointer_path.current
    } else {
        &pointer_path.old
    };
    if let Some(val) = convert(value) {
        Ok(Some(val))
    } else {
        Err(Trap::new(TrapKind::Host(Box::new(
            EnvironmentError::TypeMismatch,
        ))))
    }
}
