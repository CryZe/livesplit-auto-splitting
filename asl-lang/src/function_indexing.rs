use ast::Source;
use specs::prelude::*;

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct FunctionIndex(pub u32);

pub struct FunctionIndexing<'s>(pub &'s Source);

impl<'a, 's> System<'a> for FunctionIndexing<'s> {
    type SystemData = (WriteStorage<'a, FunctionIndex>,);

    fn run(&mut self, (mut function_indices,): Self::SystemData) {
        let mut index = 14;
        for (_, entity) in self.0.code_items() {
            function_indices
                .insert(entity, FunctionIndex(index))
                .unwrap();
            index += 1;
        }
    }
}
