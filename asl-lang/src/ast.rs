use error::{RangeError, RangeResult};
use specs::prelude::*;
use types::Ty;

#[derive(Debug)]
pub struct Source {
    pub items: Vec<Item>,
}

impl Source {
    pub fn state(&self) -> RangeResult<&State> {
        self.items
            .iter()
            .filter_map(|i| match i {
                Item::State(s) => Some(s),
                _ => None,
            }).next()
            .ok_or_else(|| RangeError::new("You need at least one state block"))
    }

    pub fn code_items<'s>(&'s self) -> impl Iterator<Item = (Option<ActionKind>, Entity)> + 's {
        self.items.iter().filter_map(|i| match i {
            Item::Action(kind, entity) => Some((Some(*kind), *entity)),
            Item::Function(entity) => Some((None, *entity)),
            _ => None,
        })
    }
}

#[derive(Debug)]
pub enum Item {
    State(State),
    Action(ActionKind, Entity),
    Function(Entity),
}

#[derive(Debug, Copy, Clone)]
pub enum ActionKind {
    Start,
    Split,
    Reset,
    IsLoading,
    GameTime,
}

#[derive(Debug)]
pub struct State {
    pub process: String,
    pub paths: Vec<PointerPath>,
}

#[derive(Debug)]
pub struct PointerPath {
    pub name: String,
    pub ty: Ty,
    pub module: String,
    pub offsets: Vec<i64>,
}

#[derive(Component)]
pub struct Children(pub Vec<Entity>);

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Scoped;

impl State {
    pub fn lookup(&self, field_name: &str) -> RangeResult<&PointerPath> {
        self.paths
            .iter()
            .find(|p| p.name == *field_name)
            .ok_or_else(|| RangeError::new(format!("Unresolved state variable '{}'", field_name)))
    }

    pub fn lookup_index(&self, field_name: &str) -> usize {
        self.paths
            .iter()
            .enumerate()
            .find(|(_, p)| p.name == *field_name)
            .unwrap_or_else(|| panic!("Unresolved state variable '{}'", field_name))
            .0
    }
}

#[derive(Debug)]
pub struct MatchCase {
    pub pattern: Option<Vec<PatternTerm>>,
    pub expr: Entity,
}

#[derive(Debug)]
pub enum PatternTerm {
    Int(i64),
    Range(i64, i64, bool),
}
