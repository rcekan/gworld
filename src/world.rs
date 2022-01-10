use super::organism::Organism;
use super::Config;
use rand::Rng;

pub struct World<E:Environs, T:Creature> {
	pub organisms :Vec<Organism<T>>,
	pub environs :E,
	fertile :Vec<usize>, // usize indexes into self.organisms 
}

pub trait Environs { // [See: docs/environs.txt]
	type Creature;
	fn new() -> Self;
}

pub trait Creature {
	type Env; // user supplied environment
	type CCT; // user supplied creature type
	fn new(env :&mut Self::Env, parents :Vec<&Self::CCT>) -> Self; // parents :&CCT
	fn rx_input(&self, input :&str, env :&Self::Env) -> f32;
	fn tx_output(&mut self, output :&str, value :f32, env :&Self::Env);
	fn act(&mut self, env :&mut Self::Env) -> f32; // returns fitness
	
//	fn make_child(&self, env :&mut Self::Env, with :Vec<&Self::CCT>) -> Self;
//	{ //, _parents :&Self::CCT) -> Self {
//		Creature::CCT.new( env )
//	}

	// let user redefine. Might want to make it some probability based on fitness and/or age, etc. 
	fn die(&self, age :usize, _fitness :f32, _env :&mut Self::Env) -> bool { 
		age > Config::get().lifespan
	}
}

impl <E:Environs<Creature = T>, T:Creature<Env=E, CCT=T>> World<E,T> {
	pub fn new() -> Self {
		let mut env = E::new();
		Self { 
			organisms: (0..Config::get().population).map(|_| Organism::new( &mut env ) ).collect(), 
			environs: env,
			fertile: Vec::new(),
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
				self.i_steps( id, &steps );
			}
			self.reproduce(&steps);
		} else {
			for _s in 0..steps { 
				self.step(); 
				self.reproduce(&1);
			}
		}
		
		if Config::log("on") {
			println!("Processing {} steps. {}", &steps, self.fitness_stats() );
		}
	}

	pub fn fitness_stats(&self) -> String {
		let (max, pop) = self.max_fitness();
		let sum = self.sum_fitness();
		format!("Fitness( sum: {:.2}, avg: {:.2}, max: {:.2} )", sum, sum/pop, max )
	}

	// not gonna lie, this doesn't really help much, in terms of speed. Maybe just delete it?
	// It's possible it helps more with very large population sizes, and more complete computations (for fitness, acting, etc)
	fn i_steps( &mut self, id :usize, steps :&usize ) {
		let org = &mut self.organisms[id];

		for _s in 0..*steps {
			if !org.alive { break; };
			org.take_step( &mut self.environs );
			// Note, one would think you could abstract all these steps in org possibly... 
		}
	}
	
	// Efficiency concern: If objects are independent of each other in the environment, 
	// We can compute all steps at once for one creature, in memory, without fear of thrashing/swapping between the potential 1000's of creatures in our environment for each step. 
	// [Ref: docs/independence-efficiency.txt]

	fn step(&mut self) {
		// now compute the outputs and act (TODO: let's do this in order of most fit)
		for org in self.organisms.iter_mut() {
			if !org.alive { continue; }
			org.take_step( &mut self.environs );
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

	fn max_fitness(&self) -> (f32, f32) { // don't confuse with org.max_fitness :/ 
		let mut max = 0.;
		let mut pop = 0.;
		for org in self.organisms.iter() {
			if !org.alive { continue; }
			max = f32::max(org.max_fitness, max);
			pop += 1.;
		};
		return (max, pop)
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
		let needed = self.offspring_needed( steps );

		// first pick the winners of offspring lottery
        let mut rng = rand::thread_rng();
		for _i in 0..needed { // self.offspring_needed( steps ) {
			
			// Pick a number, 0 - sum(org.max_fitness)
			let num = rng.gen_range(0.0.. self.sum_fitness());
			
			// Then just cycle through the org.max_fitness, 
			// until we find the "winner"
			
			let mut tot = 0.;
			for (id, org) in self.organisms.iter().enumerate() {
				if !org.alive { continue; }
				tot += f32::abs(org.max_fitness); // abs: sanity check, could shift all numbers by greatest negative number... 
				if tot >= num { // We have a winner
					self.fertile.push( id );
					break;
				}
			}
		}

		if Config::log("low") && needed>0 { println!( "Winners: {:?}", &self.fertile ); }
		
		// great, we have some babies to make!
		while self.fertile.len() > 0 {
			if let Some(id) = self.fertile.pop() {
				self.organisms[id].offspring += 1;
				let org = self.organisms[id].bud( &mut self.environs );
				// let env = self.organisms[id].environs; // pass along environs
				self.birth( org );
			}
		} // consider better reproduction strats! [see: docs/repro.txt]
	}

	fn birth(&mut self, baby :Organism<T>) {
		// Check for "dead" body
		for (id, org) in self.organisms.iter().enumerate() {
			if !org.alive & !self.fertile.contains(&id) {
				// remove it from the fertility pool (if needed). You snooze you lose. 
				// self.fertile.remove( self.fertile.iter().position(|x| *x == id).unwrap() );

				// reclaim the body
				self.organisms[id] = baby;
				return ();
			}
		}
		
		// Or add it to the end. 
		self.organisms.push( baby );
	}
}


