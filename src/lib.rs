// A library providing genome algorithms. 
use rand::Rng;

// now all other modules can access following through crate/super
pub mod math;
mod config;
mod brains;
mod organism;
mod genes;
// mod node;

pub use config::Config;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

// enum ReproductionType { 
// 	BUDDING,
// }

// Note: I wanted to put Node stuff below in a sub module... 
// But turns out everything in this library uses Node, and annoying, 
// I would have to put put(crate) in front of everything!! Wtf Rust. 
// Need a way to just say, "This whole mod is pub for library interal use"

#[derive(Copy, Clone, Debug)]
pub(crate) enum NodeType {
    INPUT,
    HIDDEN,
    OUTPUT,
}

#[derive(Copy, Clone)]
pub struct Node {
    pub(crate) idx: usize, // unique across all nodes (TODO: ... although, there's some consideration to be made for the fact that the inner neurons don't necessarily need to be a particular inner nueron, possibly we can "shift" the inner neurons as a mutation? Or we can manually shift the index for them such that various chromosomes don't overlap functions with each other's inner neurons. Not sure that's what we'd want, but it could be interesting to consider this "independence" of the inner neurons, by adjusting their assigned "idx" values algorithmically to "make sense" for a different model/constrant/design.)
    pub(crate) node_type: NodeType,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Node {
    // fn idx(&self) -> usize { self.idx as usize }
    fn is_hidden(&self) -> bool { matches!(self.node_type, NodeType::HIDDEN) }
    fn is_input(&self) -> bool { matches!(self.node_type, NodeType::INPUT) }
    fn is_output(&self) -> bool { matches!(self.node_type, NodeType::OUTPUT) }
    pub fn get_name(&self) -> String {
		Config::get().node_name( self )
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str( format!("{}", self.get_name()).as_str() )
        // f.debug_tuple( "" ).field( &self.idx ).field( &self.node_type ).finish()
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_str = match self.node_type {
            NodeType::INPUT => "I",
            NodeType::HIDDEN => "H", 
            NodeType::OUTPUT => "O", 
        };
        write!(f, "{}{}", node_str, self.idx)
    }
}

// mod organism; // do we need this pub?
use organism::Organism;
use genes::Genome;
// mod config;

pub struct World<T:Creature> {
	pub organisms :Vec<Organism<T>>,
	fertile: Vec<usize>, // list all creatures (id) to reproduce (need to check if still alive! hmm)
}

pub trait Creature {
	fn init() -> Self;
	fn calc_input(&self, input :&str) -> f32;
	fn handle_output(&mut self, output :&str, value :f32);
	fn process_result(&mut self);
	fn fitness(&self) -> f32; // , world :&World<T>) -> f32; 

	fn to_die<T:Creature>(&self, org :&Organism<T>) -> bool { 
		org.age > Config::get().lifespan
		// ultimately, might want it some probabalistic based on a continuous fit function. 
	}
}

// DEFINED ABOVE    : 
// pub struct World<T:Creature> {
// 	pub organisms :Vec<Organism<T>>,
// 	fertile: Vec<usize>, // list all creatures (g_id) to reproduce (need to check alive! hmm)
// }

impl<T:Creature> World<T> {
	pub fn new() -> Self {
		Self { 
			organisms: (0..Config::get().population).map(|_| Organism::new() ).collect(), 
			fertile: Vec::new(), 
		}
	}

	fn birth(&mut self, genome :Genome) {
		let mut reclaim = -1;
		for (id, org) in self.organisms.iter().enumerate() {
			if !org.alive {
				reclaim = id as isize;
				break;
			}
		}
		
		let org = Organism::from_genome( genome );
		
		if reclaim >= 0 {
			let id = reclaim as usize;
			self.organisms[id] = org;
		} else { // or else add to the back. 
			self.organisms.push( org );
		}
	}

	// live carries out processes for all creatures
	pub fn live(&mut self) { 
		self.advance( self.avg_life().floor() as usize ); 
	}

	pub fn advance(&mut self, total_steps :usize) {
		// Independent lives will reproduce at the end of their lives (or step sequences, if you will), for the sake of computational loops and efficiency. We need handle their cycles a little differently.
		if Config::get().independent {
			let avg = self.avg_life().floor() as usize;
			let mut steps = 0;
			// let's break loops up into life-span size chunks
			while steps < total_steps { // this way reproduction can occur
				self.steps( usize::min( avg, total_steps-steps) );
				steps += avg; // we may overshoot here, math still works out
			}
		} else {
			self.steps( total_steps )
		}
	}

	// All control comes through steps. 
	fn steps(&mut self, steps :usize) {
		if Config::get().independent {
			// assert!( steps <= Config::get().avg_life() ); 
			// we don't really need this now. Let's just cap reproduction at population size.
			for id in 0 .. self.organisms.len() { // iter().enumerate() {
				// if !genome.alive { continue }
				self.independent_steps( id, &steps );
			}
			self.reproduce(&steps);
		} else {
			for _s in 0..steps { 
				self.step(); 
				self.reproduce(&1);
			}
		}
		println!("Processing {} steps. {}, {}", &steps, self.sum_fitness(), self.max_fitness() );

	}		

	fn avg_life(&self) -> f32 {
		Config::get().lifespan as f32
	}

	fn offspring_needed(&self, steps :&usize) -> usize {
		let mut needed = (steps * Config::get().population ) as f32 / self.avg_life();
        let mut rng = rand::thread_rng();

		// fraction determines probability of additional child.
		let fraction = needed - needed.floor();
		if fraction >= rng.gen_range(0.0..1.0) { 
			needed += 1.; // yay a bonus child!
		}
		needed.floor() as usize
	 }

	fn max_fitness(&self) -> f32 {
		let mut max = 0.;
		for org in self.organisms.iter() {
			if !org.alive { continue; }
			max = f32::max(org.max_fitness, max);
		};
		return max
	}
		
	fn sum_fitness(&self) -> f32 {
		let mut tot = 0.;
		for org in self.organisms.iter() {
			if !org.alive { continue; }
			tot += f32::abs(org.max_fitness); // avoid negative fitness possibiities
		};
		return f32::max(tot, 0.001);
	}

	fn reproduce(&mut self, steps :&usize) {
		// first pick the winners of offspring lottery
        let mut rng = rand::thread_rng();
		for _i in 0..self.offspring_needed( steps ) {
			
			// Pick a number, 0 - sum(g.max_fitness)
			let num = rng.gen_range(0.0.. self.sum_fitness());
			
			// Then just cycle throug the genomes.max_fitness(), 
			// until we find the "winner"
			
			let mut tot = 0.;
			for (id, org) in self.organisms.iter().enumerate() {
				if !org.alive { continue; }
				tot += f32::abs(org.max_fitness);
				if tot >= num {
					// We have a winner
					self.fertile.push( id );
					// print!("Winners: {:?}, ", &self.fertile);
					break;
				}
			}
		}

		if self.fertile.len() < 1 { return (); }
		
		// great, we have some babies to make!
		while self.fertile.len() > 0 {
			if let Some(id) = self.fertile.pop() {
				self.organisms[id].offspring += 1;
				let genome = self.organisms[id].genome.bud();
				self.birth( genome );
			}
		} // consider better reproduction strats! [see: docs/repro.txt]
		
		// Just print it out. 
		// println!( "Winners: {:?}", &self.fertile );
	}

	// cannot borrow self as mutable more than one time. 
	fn independent_steps( &mut self, id :usize, steps :&usize ) {
	// This genome/creature/brain bullshit seems to be gunking things up. 
	// Do I really want fine control over mutability of each separately?
	// It seems it's making more problems than it's worth!

		let org = &mut self.organisms[id];
		if !org.alive { return (); } // sanity check

		'step_loop: for _s in 0..*steps {
			org.get_inputs();
			org.process_inputs();
			org.set_outputs();
			
			org.creature.process_result();

			// Won't allow this! Because I borrow against self too many times. 
			// So much for the reusability of the age_genome function. 
			//	for _s in 0..*steps {
			//		self.age_genome( g_id.clone(), 1 );
			//	}
			
			org.max_fitness = f32::max( org.max_fitness, org.creature.fitness() );

			org.age( 1 ); // note: Death may occur from this process. 
			if !org.alive { break 'step_loop; };
		}

	}
	
	// Efficiency concern: If objects are independent of each other in the environment, 
	// We can compute all steps at once for one creature, in memory, without fear of thrashing/swapping between the potential 1000's of creatures in our environment for each step. 
	// more: [docs/independence-efficiency.txt]

	fn step(&mut self) {
		// first collect the inputs
        for org in self.organisms.iter_mut() {
			if !org.alive { continue; }
			org.get_inputs( );
		}

		// now compute the outputs (and do stuff)
		for org in self.organisms.iter_mut() {
			if !org.alive { continue; }

			org.process_inputs(); // also squashes inner nodes
			org.set_outputs();
			org.creature.process_result();

			// and update max_fitness while we're here: 
			org.max_fitness = f32::max( org.max_fitness, org.creature.fitness() );
        }

		// And finally time to die
		for org in self.organisms.iter_mut() {
			org.age( 1 );
		}
		// self.expunge_dead(); // [see: docs/expunge.txt]
	}
}

// Got sick of passing around the config everywhere. Let's try to make it a global variable. 
use lazy_static::lazy_static;
use std::sync::{Mutex}; // , MutexGuard};

lazy_static!{
    static ref CONFIG :Mutex<Config> = Mutex::new( Config {
		population: 100,
		lifespan: 50,
		genome_size: 50,
		use_chromo: true,
		independent: false,
		strength_mult: 4.0, // multiplier for gene strengths
		// nodes: (5, 3, 5),
		inputs: Vec::new(),
		outputs: Vec::new(),
		neurons: 5,
	});
}


// Also, gotta give a shout out to genevo. It looks to be an excellent library. I was really considering using it, and I still feel like I _should_ be building upon it. 
// However a couple points:
// 1) I'm adding this concept of chromosome, and plan to explore diploid/haploid etc, types of breedings, among other things
// 2) So it seems like it'll be easier to tinker if I just do this from the ground up as opposed to trying to fit genevo's model.
// 3) I just want to learn Rust, and the reasons behind generics and traits, and how to create libraries, and to contribute my code.
// 
// I hope you like it. :)
// (And no I won't use rustfmt just yet. I actually _like_ the current formatting choices. :/ )
