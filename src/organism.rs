use super::genes::Genome; // we need pub here to re-export, right?
use super::brains::Brain;
use super::{math, Creature};
// pub mod math;

// Is there some way to make this whole fuckin file pub(crate), while letting the regular pub override that where noted?

pub struct Organism <T:Creature> { 
	pub alive: bool,
	pub age: usize,
	pub(crate) offspring: usize,
	pub(crate) max_fitness: f32,

	pub(crate) genome :Genome,
	pub(crate) brain :Brain,
	pub creature :T,
}

impl <T:Creature> Organism <T> { 
	pub(crate) fn new() -> Self {
		let genome = Genome::new();
		Organism::from_genome( genome )
	}

	// that's fascinating.
	// reproduction is something the world does.
	// not that the organism does. It takes 2 to reproduce, (or more!). 
	// It's never going to work 

	pub(crate) fn from_genome( genome :Genome ) -> Self {
		Self {
			brain: Brain::new( &genome ), // need to build brain first appartently (oh rust)
			genome, 
			creature: T::init(),

			alive: true, age: 0, offspring: 0, max_fitness: 0.,
		}
	}

	pub(crate) fn age( &mut self, steps :usize ) {
		self.age += steps;
		if self.creature.to_die( self ) {
			self.alive = false;
		}
	}
	
	pub(crate) fn set_outputs( &mut self ) {
		for nuron in self.brain.outputs.iter() {
			self.creature.handle_output( nuron.node.get_name().as_str(), nuron.state );
		}
	}

	pub(crate) fn get_inputs( &mut self ) {
		for node in self.brain.inputs.iter() {
			self.brain.node_state[ node.idx ] = self.creature.calc_input( &node.get_name() );
		}
	}

    pub(crate) fn process_inputs( &mut self ) {
		// first sum all nodes in hidden and output layer, using node_state from last live() iteration, plus with updated inputs of course
		for nuron in self.brain.nurons.iter_mut().chain( self.brain.outputs.iter_mut() ) {
			let mut state:f32 = 0f32;
			for gene in nuron.incoming.iter() {
				let s:f32 = gene.strength;
				let value = self.brain.node_state[ gene.source.idx ];
				state += value * s;
			}
        	nuron.state = state;
		}

		// now write the values to the node_state
		for nuron in self.brain.nurons.iter() {
			self.brain.node_state[ nuron.node.idx ] = math::tanh( nuron.state ); // squash them down!
		}
		
		//// not really necessary... but we're doing it for completeness, maybe someone will expect it to be there in the future. :|
		//for nuron in outputs.iter() {
		//	node_state[ nuron.root.idx() ] = nuron.state; // don't squash outputs. We'll manually 
		//}
    }
}

