use ast::{self, Source};
use code_gen::CodeGenDesc;
use debug_info::SrcByteRange;
use error::{RangeError, RangeResult, ResultExt};
use name_resolution::Vars;
use specs::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Component)]
pub enum Ty {
    Unit,
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Int,
    Float,
    Number,
    Bits, // TODO Consider general union types
}

#[derive(Component)]
pub struct TypeChecking(pub Vec<Inference>);

pub enum Inference {
    SameAsMe(Entity),
    VarSameAsMe(usize),
    StateVarSameAsMe(String),
    TypeHint(Entity),
}

pub struct TypeSystem<'s> {
    src: &'s Source,
    result: RangeResult<()>,
}

impl<'s> TypeSystem<'s> {
    pub fn new(src: &'s Source) -> Self {
        Self {
            src,
            result: Ok(()),
        }
    }
    pub fn run(mut self, world: &World) -> RangeResult<()> {
        self.run_now(&world.res);
        self.result
    }
}

type SystemData<'a> = (
    Entities<'a>,
    ReadStorage<'a, TypeChecking>,
    WriteStorage<'a, Ty>,
    ReadStorage<'a, Vars>,
    ReadStorage<'a, SrcByteRange>,
);

impl<'a, 's> System<'a> for TypeSystem<'s> {
    type SystemData = SystemData<'a>;

    fn run(&mut self, sys_data: SystemData) {
        self.result = try_run(&self.src, sys_data);
    }
}

fn try_run(
    src: &Source,
    (entities, type_checking, mut types, vars, ranges): SystemData,
) -> RangeResult<()> {
    let state = src.state()?;

    loop {
        let mut is_dirty = false;

        run_once(
            &state,
            &mut is_dirty,
            &entities,
            &type_checking,
            &mut types,
            &vars,
            &ranges,
        )?;

        if !is_dirty {
            let is_now_dirty = apply_hints(&entities, &type_checking, &mut types);
            if !is_now_dirty {
                break;
            }
        }
    }

    Ok(())
}

fn apply_hints(
    entities: &Entities,
    type_checking: &ReadStorage<TypeChecking>,
    types: &mut WriteStorage<Ty>,
) -> bool {
    let mut is_dirty = false;

    for (me, TypeChecking(inference_list)) in (&**entities, type_checking).join() {
        if let Some(my_ty) = types.get(me).cloned() {
            for inference in inference_list {
                if let Inference::TypeHint(other) = inference {
                    let set = {
                        let other_ty = types.get(*other);
                        if let Ok(Some(_)) = spread(other_ty, Some(&my_ty), &mut is_dirty) {
                            true
                        } else {
                            false
                        }
                    };
                    if set {
                        let _ = types.insert(*other, my_ty.clone());
                    }
                }
            }
        }
    }

    is_dirty
}

fn run_once(
    state: &ast::State,
    is_dirty: &mut bool,
    entities: &Entities,
    type_checking: &ReadStorage<TypeChecking>,
    types: &mut WriteStorage<Ty>,
    vars: &ReadStorage<Vars>,
    ranges: &ReadStorage<SrcByteRange>,
) -> RangeResult<()> {
    for (me, TypeChecking(inference_list)) in (&**entities, type_checking).join() {
        let mut inner_is_dirty = false;
        let mut my_ty = types.get(me).cloned();

        for inference in inference_list {
            match inference {
                Inference::SameAsMe(other) => {
                    let other_ty = types.get(*other);
                    if let Some(ty) = spread(my_ty.as_ref(), other_ty, &mut inner_is_dirty)
                        .with_entity_range(me, ranges)?
                    {
                        my_ty = Some(ty.clone());
                    }
                }
                Inference::VarSameAsMe(var_id) => {
                    let var = vars.get(me).unwrap().0[*var_id];
                    let other_ty = types.get(var);
                    if let Some(ty) = spread(my_ty.as_ref(), other_ty, &mut inner_is_dirty)
                        .with_entity_range(me, ranges)?
                    {
                        my_ty = Some(ty.clone());
                    }
                }
                Inference::StateVarSameAsMe(field_name) => {
                    let path = state.lookup(field_name).with_entity_range(me, ranges)?;
                    if let Some(ty) = spread(my_ty.as_ref(), Some(&path.ty), &mut inner_is_dirty)
                        .with_entity_range(me, ranges)?
                    {
                        my_ty = Some(ty.clone());
                    }
                }
                Inference::TypeHint(_) => {
                    // Type Hinting has very low priority, so we only do it once
                    // we normally would finish the type inference.
                }
            }
        }

        if inner_is_dirty {
            if let Some(my_ty) = my_ty {
                for inference in inference_list {
                    match inference {
                        Inference::SameAsMe(other) => {
                            let _ = types.insert(*other, my_ty.clone());
                        }
                        Inference::VarSameAsMe(var_id) => {
                            let var = vars.get(me).unwrap().0[*var_id];
                            let _ = types.insert(var, my_ty.clone());
                        }
                        Inference::StateVarSameAsMe(_) => {
                            // No need to back propagate to a state
                            // variable, as it already is fully typed.
                        }
                        Inference::TypeHint(_) => {
                            // Same as above, we don't do type hinting during
                            // the type propagation.
                        }
                    }
                }
                let _ = types.insert(me, my_ty);
                *is_dirty = true;
            }
        }
    }

    Ok(())
}

pub struct CheckForUnassignedTypes;

impl<'a> System<'a> for CheckForUnassignedTypes {
    type SystemData = (ReadStorage<'a, Ty>, ReadStorage<'a, CodeGenDesc>);

    fn run(&mut self, (types, code_gen_descs): Self::SystemData) {
        for _ in (&code_gen_descs, !&types).join() {
            panic!("Failed to infer all types");
        }
    }
}

fn inner_spread<'a>(a: Option<&Ty>, b: Option<&'a Ty>) -> RangeResult<Option<&'a Ty>> {
    let (a, b) = match (a, b) {
        (Some(a), Some(b)) => (a, b),
        (None, Some(a)) => return Ok(Some(a)),
        _ => return Ok(None),
    };
    Ok(match (a, b) {
        (Ty::Int, x) if x.is_specific_int() => Some(x),
        (Ty::Float, x) if x.is_specific_float() => Some(x),
        (Ty::Number, x) if x.is_more_specific_number() => Some(x),
        (Ty::Number, Ty::Bits) | (Ty::Bits, Ty::Number) => Some(&Ty::Int),
        (Ty::Bits, x) if x.is_more_specific_bits_type() => Some(x),
        (x, Ty::Int) if x.is_specific_int() => None,
        (x, Ty::Float) if x.is_specific_float() => None,
        (x, Ty::Number) if x.is_more_specific_number() => None,
        (x, Ty::Bits) if x.is_more_specific_bits_type() => None,
        (a, b) if a == b => None,
        _ => {
            return Err(RangeError::new(format!(
                "Type conflict between {} and {}",
                a, b
            )))
        }
    })
}

fn spread<'a>(
    a: Option<&Ty>,
    b: Option<&'a Ty>,
    is_dirty: &mut bool,
) -> RangeResult<Option<&'a Ty>> {
    let new_val = inner_spread(a, b)?;
    *is_dirty |= a != b;
    Ok(new_val)
}

impl Ty {
    fn is_specific_int(&self) -> bool {
        use self::Ty::*;
        match self {
            U8 | U16 | U32 | U64 | I8 | I16 | I32 | I64 => true,
            _ => false,
        }
    }

    fn is_more_specific_number(&self) -> bool {
        use self::Ty::*;
        match self {
            U8 | U16 | U32 | U64 | I8 | I16 | I32 | I64 | F32 | F64 | Float | Int => true,
            _ => false,
        }
    }

    fn is_more_specific_bits_type(&self) -> bool {
        use self::Ty::*;
        match self {
            U8 | U16 | U32 | U64 | I8 | I16 | I32 | I64 | Int | Bool => true,
            _ => false,
        }
    }

    fn is_specific_float(&self) -> bool {
        use self::Ty::*;
        match self {
            F32 | F64 => true,
            _ => false,
        }
    }

    pub fn is_uint(&self) -> bool {
        use self::Ty::*;
        match self {
            U8 | U16 | U32 | U64 => true,
            _ => false,
        }
    }
}

use std::fmt;

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ty::Unit => write!(f, "unit"),
            Ty::Bool => write!(f, "bool"),
            Ty::U8 => write!(f, "u8"),
            Ty::U16 => write!(f, "u16"),
            Ty::U32 => write!(f, "u32"),
            Ty::U64 => write!(f, "u64"),
            Ty::I8 => write!(f, "i8"),
            Ty::I16 => write!(f, "i16"),
            Ty::I32 => write!(f, "i32"),
            Ty::I64 => write!(f, "i64"),
            Ty::F32 => write!(f, "f32"),
            Ty::F64 => write!(f, "f64"),
            Ty::Int => write!(f, "{{int}}"),
            Ty::Float => write!(f, "{{float}}"),
            Ty::Number => write!(f, "{{number}}"),
            Ty::Bits => write!(f, "{{bits}}"),
        }
    }
}
