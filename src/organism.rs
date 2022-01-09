use super::genes::Genome; // we need pub here to re-export, right?
use super::brains::Brain;
use super::{math, Creature};

// Is there some way to make this whole fuckin file pub(crate), while letting the regular pub override that where noted?

// enum ReproductionType { 
// 	BUDDING,
// }

pub struct Organism <T:Creature> { 
	// pub environs: &'a E,
	pub alive: bool,
	pub age: usize,
	pub(crate) offspring: usize,
	pub(crate) max_fitness: f32,
	pub(crate) fitness: f32,

	pub(crate) genome :Genome,
	pub(crate) brain :Brain,
	pub creature :T,
}

impl <T:Creature + Creature<CCT = T>> Organism <T> { 
	pub(crate) fn new( env :&mut T::Env ) -> Self {
		let genome = Genome::new();
		Organism::from_genome( genome, env, Vec::new() )
	}

	// that's fascinating.
	// reproduction is something the world does.
	// not that the organism does. It takes 2 to reproduce, (or more!). 
	// It's never going to work 

	pub(crate) fn from_genome( genome :Genome, env :&mut T::Env, parents :Vec<&T::CCT> ) -> Self {
		Self {
			//environs: env,
			brain: Brain::new( &genome ), // need to build brain first appartently (oh rust)
			genome, 
			creature: T::new( env, parents ),

			alive: true, age: 0, offspring: 0, 
			fitness: 0., max_fitness: 0.,
		}
	}

	pub(crate) fn bud( &self, env :&mut T::Env ) -> Self {
		let mut parents = Vec::new();
		parents.push( &self.creature );
		return Self::from_genome( self.genome.bud(), env, parents )
	}

// 	pub(crate) fn handle_result( &mut self, env :&T::Env ) {
// 		self.set_outputs( env );
// 	}

	pub(crate) fn take_step( &mut self, env :&mut T::Env ) {
		self.get_inputs( env );
		self.process_inputs();
		self.set_outputs( env );
		self.take_action( env );
	}

	pub(crate) fn take_action( &mut self, env :&mut T::Env ) {
		self.fitness = f32::max( 0.0001, self.creature.act( env )); // no negative fitness (for now, see note in World.reproduce)
		self.max_fitness = f32::max( 
			self.max_fitness, 
			self.fitness,
		);
		self.age( 1, env );
	}

	// may cause death (is steps really necessary??)
	fn age( &mut self, steps :usize, env :&mut T::Env ) {
		self.age += steps;
		// println!("Aging: {}", self.age);
		if self.creature.die( self.age, self.fitness, env ) {
			// println!("Create is dieing");
			self.alive = false;
		}
	}

	pub(crate) fn set_outputs( &mut self, env :&T::Env ) {
		for nuron in self.brain.outputs.iter() {
			self.creature.tx_output( nuron.node.get_name().as_str(), nuron.state, env );
		}
	}

	pub(crate) fn get_inputs( &mut self, env :&T::Env ) {
		for node in self.brain.inputs.iter() {
			self.brain.node_state[ node.idx ] = self.creature.rx_input( &node.get_name(), env );
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

