// A library to enable genetically-inspired breeding algorithms. 

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
    #[test] // Anybody want to take a stab at it? Code testing is highly desirable for interviews I've heard. wink wink
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

// MISSION
// The goal of this library is to make building genetic algorithms intuitive. 
// Most importantly, I'm focusing on the abstractions, the implementations may change,
// but the goal is to reduce boilerplate, and provide a DRY environment so the end-user can focus on the parts that matter. 
// Of course, ultimately, there are (and will be) things that are yet to be provided in this library. 
// Things where the user will want to adapt/alter the innerworkings of the library, 
// It's highly encouraged, and please let us know and we can incorporate it into the base code if you'd be so generous. :]
// I'm excited to keep working on it, and I hope at least someone else out there may experience the joy of working with genetic algorithms. 
// It's so much fun to see something grow and evolve, and it's up to you to provide healthy inputs and outputs. 
// That's the true art of making genetic algorithms, in my experience. Really think about your inputs and outputs. 
// 
// Also, gotta give a shout out to genevo. It looks to be an excellent library. I was really considering using it, and I still feel like I _should_ be building upon it. 
// However a couple points:
// 0) Continuous fit function
// 1) I'm adding this concept of chromosome, and plan to explore diploid/haploid etc, types of breedings, among other things
// 2) Multiple fit functions (still in the works)
// 3) It seems like it'll be easier to tinker with stuff if I just do it from the ground up as opposed to trying to fit genevo's model. I may yet retrofit it though, if that ever seems prudent. 
// 4) I want to learn both rust and genetic algorithms. So this is a lot of fun for me. :D
// 
// I hope you like it! :)
//
//
// PS.
// (And no I won't use rustfmt just yet. I actually _like_ the current formatting choices. :/ )
// When setting a value, you use:  value_name: value, 
// and then when defining a type:  value_name :Type
// It helps me read the code better* this way. You're doing two very symantically different things, why not have different syntax for it? 
// * (The syntax adopted here helps create more space around the names of the variables, so they "pop" out visibly when one reads the code, especially true with a long argument list.)
// But that's the beauty of Rust and rustfmt. You can just apply it on your end and everybody's happy! No artistic freedoms compromised. ;)
