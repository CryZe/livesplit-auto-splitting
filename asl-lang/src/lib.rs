#![allow(unknown_lints)]

#[macro_use]
extern crate lalrpop_util;
extern crate specs;
#[macro_use]
extern crate specs_derive;

pub extern crate parity_wasm;

mod ast;
mod code_gen;
mod debug_info;
mod error;
mod function_indexing;
mod function_signatures;
mod name_resolution;
mod ops;
mod reg_alloc;
mod reg_extend;
mod specify_general_types;
mod types;

lalrpop_mod!(
    #[allow(clippy)]
    grammar
);

use debug_info::SrcByteRange;
pub use debug_info::{Hover, Span};
pub use error::{Error, Result};
use lalrpop_util::ParseError;
pub use parity_wasm::elements::Module;
use specs::prelude::*;
pub use types::Ty;

fn create_world() -> World {
    let mut world = World::new();
    world.register::<ast::Children>();
    world.register::<ast::Scoped>();
    world.register::<code_gen::CodeGenDesc>();
    world.register::<debug_info::ReferencesVar>();
    world.register::<debug_info::SrcByteRange>();
    world.register::<function_indexing::FunctionIndex>();
    world.register::<function_signatures::FunctionCall>();
    world.register::<function_signatures::FunctionSignatureRegisters>();
    world.register::<name_resolution::DeclaredBy>();
    world.register::<name_resolution::FunctionDecl>();
    world.register::<name_resolution::FunctionParamAsVar>();
    world.register::<name_resolution::VarDecl>();
    world.register::<name_resolution::VarNames>();
    world.register::<name_resolution::Vars>();
    world.register::<reg_alloc::FunctionRegisters>();
    world.register::<reg_alloc::Registers>();
    world.register::<reg_extend::InferExtending>();
    world.register::<reg_extend::NeedsExtending>();
    world.register::<types::Ty>();
    world.register::<types::TypeChecking>();

    world
}

fn parse(world: &mut World, src: &str) -> Result<ast::Source> {
    match grammar::SourceParser::new().parse(world, src) {
        Ok(s) => Ok(s),
        Err(ParseError::InvalidToken { location }) => Err(Error {
            message: String::from("Invalid token"),
            span: Some(Span::from_byte_position(src, location)),
        }),
        Err(ParseError::UnrecognizedToken {
            token: Some((l, token, r)),
            expected,
        }) => Err(Error {
            message: format!(
                "Unexpected token \"{}\", expected one of: {}",
                token,
                expected.join(", ")
            ),
            span: Some(SrcByteRange(l, r).to_span(src)),
        }),
        Err(ParseError::UnrecognizedToken {
            token: None,
            expected,
        }) => Err(Error {
            message: format!(
                "Unexpected end of file, expected one of: {}",
                expected.join(", ")
            ),
            span: Some(Span::from_byte_position(src, src.len())),
        }),
        Err(ParseError::ExtraToken {
            token: (l, token, r),
        }) => Err(Error {
            message: format!("Encountered an extra token \"{}\"", token),
            span: Some(SrcByteRange(l, r).to_span(src)),
        }),
        _ => unimplemented!(),
    }
}

fn base_passes(world: &mut World, src: &str, source: &ast::Source) -> Result<()> {
    name_resolution::NameResolution::new(source)
        .run(&world)
        .map_err(|e| e.spanned(src))?;

    function_signatures::VerifyFunctionCallSignatures::new()
        .run(&world)
        .map_err(|e| e.spanned(src))?;

    types::TypeSystem::new(source)
        .run(&world)
        .map_err(|e| e.spanned(src))?;

    types::CheckForUnassignedTypes.run_now(&world.res);

    Ok(())
}

pub fn compile(src: &str) -> Result<Module> {
    let mut world = create_world();
    let source = parse(&mut world, src)?;

    base_passes(&mut world, src, &source)?;

    reg_extend::RegisterExtensionInference.run_now(&world.res);
    specify_general_types::SpecifyGeneralTypes.run_now(&world.res);
    function_indexing::FunctionIndexing(&source).run_now(&world.res);
    function_signatures::AllocFunctionSignatureRegisters.run_now(&world.res);
    reg_alloc::RegAlloc(&source).run_now(&world.res);
    let mut code_gen = code_gen::CodeGen(&source, None);
    code_gen.run_now(&world.res);

    Ok(code_gen.1.unwrap())
}

pub fn hover(src: &str, line: usize, column: usize) -> Result<Option<Hover>> {
    let mut world = create_world();
    let source = parse(&mut world, src)?;

    base_passes(&mut world, src, &source)?;
    Ok(debug_info::HoverSystem::new(src, line, column).run(&world))
}

pub fn go_to_definition(src: &str, line: usize, column: usize) -> Result<Option<Span>> {
    let mut world = create_world();
    let source = parse(&mut world, src)?;

    base_passes(&mut world, src, &source)?;
    Ok(debug_info::GoToDefinition::run(src, &world, line, column).map(|(_, s)| s))
}

pub fn find_all_references(src: &str, line: usize, column: usize) -> Result<Option<Vec<Span>>> {
    let mut world = create_world();
    let source = parse(&mut world, src)?;

    base_passes(&mut world, src, &source)?;
    Ok(debug_info::FindAllVariableReferences::run(
        src, &world, line, column,
    ))
}

#[cfg(test)]
mod tests;
