// A library providing genetically-inspired breeding algorithms. 

// now all other modules can access following through crate/super
pub mod math;
mod world;
mod config;
mod brains;
mod organism;
mod genes;
mod node;

pub use world::{World, Creature, Environs};
pub use config::Config; 

#[cfg(test)]
mod tests { // Yeah I need to do this. 
    #[test] // Anybody want to take a stab at it? Code testing is highly desirable for interviews I've found. wink wink
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}


// Also, gotta give a shout out to genevo. It looks to be an excellent library. I was really considering using it, and I still feel like I _should_ be building upon it. 
// However a couple points:
// 1) I'm adding this concept of chromosome, and plan to explore diploid/haploid etc, types of breedings, among other things
// 2) So it seems like it'll be easier to tinker if I just do this from the ground up as opposed to trying to fit genevo's model.
// 3) I want to learn both rust and genetic algorithms. So this is a lot of fun for me. :D
// 
// I hope you like it. :)
// (And no I won't use rustfmt just yet. I actually _like_ the current formatting choices. :/ )
