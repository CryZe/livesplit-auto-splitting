use specs::prelude::*;
use types::Ty;

pub struct SpecifyGeneralTypes;

impl<'a> System<'a> for SpecifyGeneralTypes {
    type SystemData = WriteStorage<'a, Ty>;

    fn run(&mut self, mut types: Self::SystemData) {
        // Since this runs after type inference, we can't do anything fancy here
        // based on the values of integer literals or so. That would need to
        // either be integrated into the type inference or be a pre-pass. So
        // this will always have to stay this simple.
        for ty in (&mut types).join() {
            match ty {
                Ty::Int | Ty::Number => *ty = Ty::I32,
                Ty::Float => *ty = Ty::F64,
                Ty::Bits => unreachable!("The Bits type shouldn't get past the type inference"),
                _ => {}
            }
        }
    }
}
