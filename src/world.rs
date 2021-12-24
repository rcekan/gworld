use super::organism::Organism;
use super::Config;
use rand::Rng;

pub struct World<E:Environs, T:Creature> {
	pub organisms :Vec<Organism<T>>,
	pub environs :E,
	fertile :Vec<usize>, // usize indexes into self.organisms 
}

pub trait Environs {
	type Creature;
	fn inputs_for( &mut self, creature :&Self::Creature ) -> bool;
	fn init() -> Self;
}

pub trait Creature {
	fn init() -> Self;
	fn calc_input(&self, input :&str) -> f32;
	fn handle_output(&mut self, output :&str, value :f32);
	fn process_result(&mut self);
	fn fitness(&self) -> f32; // , world :&World<T>) -> f32; 
	
	fn to_die<T:Creature>(&self, org :&Organism<T>) -> bool { 
		org.age > Config::get().lifespan
		// ultimately, might want it some probability based on a continuous fit-function derivative. 
	}
}

impl <E:Environs<Creature = T>, T:Creature> World<E,T> {
	pub fn new() -> Self {
		Self { 
			organisms: (0..Config::get().population).map(|_| Organism::new() ).collect(), 
			environs: E::init(), 
			fertile: Vec::new(),
		}
	}

	fn birth(&mut self, org :Organism<T>) {
		let mut reclaim = -1; // Check for "dead" body
		for (id, org) in self.organisms.iter().enumerate() {
			if !org.alive {
				// remove it from the fertility pool (if needed). You snooze you lose. 
				// self.fertile.remove( self.fertile.iter().position(|x| *x == id).unwrap() );

				// and then reclaim the body
				reclaim = id as isize;
				break;
			}
		}
		
		// Overwrite the "dead" organism
		if reclaim >= 0 { 
			let id = reclaim as usize;
			self.organisms[id] = org;
		} else { 
		// Or add it to the end. 
			self.organisms.push( org );
		}
	}

	// The main loop sequence. Processes in chunks equal to avg_life
	pub fn live(&mut self) { 
		self.advance( self.avg_life().floor() as usize ); 
	}

	pub fn advance(&mut self, total_steps :usize) {
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
			// It's okay, user can use how they want. 
			// We'll just cap reproduction at population size.
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
		
		if Config::log("on") {
			println!("Processing {} steps. {}, {}", &steps, self.sum_fitness(), self.max_fitness() );
		}
	}

	fn independent_steps( &mut self, id :usize, steps :&usize ) {
		let org = &mut self.organisms[id];
		if !org.alive { return (); } // sanity check

		'step_loop: for _s in 0..*steps {
			org.get_inputs();
			org.process_inputs();
			org.set_outputs();
			
			org.creature.process_result();
			org.max_fitness = f32::max( org.max_fitness, org.creature.fitness() );

			org.age( 1 ); // note: Death may occur from this process. 
			if !org.alive { break 'step_loop; };

			// self.environs.inputs_for( &org.creature ); // testing
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

	// Reproduction stuff ===========================================

	fn avg_life(&self) -> f32 {
		Config::get().lifespan as f32
	}

	// The fraction of the population that was depleted during these steps (based on avg_life)
	fn offspring_needed(&self, steps :&usize) -> usize {
		let mut needed = (steps * Config::get().population ) as f32 / self.avg_life();
        let mut rng = rand::thread_rng();

		// fraction determines probability of additional child.
		let fraction = needed - needed.floor();
		if fraction >= rng.gen_range(0.0..1.0) { 
			needed += 1.; // yay a bonus child!
		}
		usize::min( needed.floor() as usize, Config::get().population ) // cap it at population size
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
			
			// Pick a number, 0 - sum(org.max_fitness)
			let num = rng.gen_range(0.0.. self.sum_fitness());
			
			// Then just cycle through the org.max_fitness(), 
			// until we find the "winner"
			
			let mut tot = 0.;
			for (id, org) in self.organisms.iter().enumerate() {
				if !org.alive { continue; }
				tot += f32::abs(org.max_fitness);
				if tot >= num { // We have a winner
					self.fertile.push( id );
					break;
				}
			}
		}

		if Config::log("low") { println!( "Winners: {:?}", &self.fertile ); }

		if self.fertile.len() < 1 { return (); }
		
		// great, we have some babies to make!
		while self.fertile.len() > 0 {
			if let Some(id) = self.fertile.pop() {
				self.organisms[id].offspring += 1;
				let org = self.organisms[id].bud();
				// let env = self.organisms[id].environs; // pass along environs
				self.birth( org );
			}
		} // consider better reproduction strats! [see: docs/repro.txt]
		
	}
}


