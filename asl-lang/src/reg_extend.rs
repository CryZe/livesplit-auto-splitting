use name_resolution::Vars;
use specs::prelude::*;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct NeedsExtending;

#[derive(Component)]
pub struct InferExtending(pub Vec<ExtendConnection>);

pub enum ExtendConnection {
    FromEntity(Entity),
    LoadVar(usize),
    StoreVar(usize),
}

pub struct RegisterExtensionInference;

impl<'a> System<'a> for RegisterExtensionInference {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, NeedsExtending>,
        ReadStorage<'a, InferExtending>,
        ReadStorage<'a, Vars>,
    );

    fn run(&mut self, (entities, mut needs_extending, infer_extending, vars): Self::SystemData) {
        use self::ExtendConnection::*;

        loop {
            let mut is_dirty = false;
            for (me, infer_extending) in (&*entities, &infer_extending).join() {
                for connection in &infer_extending.0 {
                    match connection {
                        FromEntity(entity) => {
                            if needs_extending.get(*entity).is_some() {
                                is_dirty |= needs_extending
                                    .insert(me, NeedsExtending)
                                    .unwrap()
                                    .is_none();
                            }
                        }
                        LoadVar(id) => {
                            let var = vars.get(me).unwrap().0[*id];
                            if needs_extending.get(var).is_some() {
                                is_dirty |= needs_extending
                                    .insert(me, NeedsExtending)
                                    .unwrap()
                                    .is_none();
                            }
                        }
                        StoreVar(id) => {
                            let var = vars.get(me).unwrap().0[*id];
                            if needs_extending.get(me).is_some() {
                                is_dirty |= needs_extending
                                    .insert(var, NeedsExtending)
                                    .unwrap()
                                    .is_none();
                            }
                        }
                    }
                }
            }
            if !is_dirty {
                break;
            }
        }
    }
}
