// A library providing genome algorithms. 
// Note: This library makes more sense when read from the bottom up. It's just how I roll. Sorry not sorry.
use rand::Rng;
pub mod math;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

#[derive(Copy, Clone, Debug)]
enum NodeType {
    INPUT,
    HIDDEN,
    OUTPUT,
}

#[derive(Copy, Clone)]
pub struct Node {
    idx: usize, // unique across all nodes (TODO: ... although, there's some consideration to be made for the fact that the inner neurons don't necessarily need to be a particular inner nueron, possibly we can "shift" the inner neurons as a mutation? Or we can manually shift the index for them such that various chromosomes don't overlap functions with each other's inner neurons. Not sure that's what we'd want, but it could be interesting to consider this "independence" of the inner neurons, by adjusting their assigned "idx" values algorithmically to "make sense" for a different model/constrant/design.)
    node_type: NodeType,
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

impl std::fmt::Display for Gene {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let active = if self.active { 'T' } else { 'F' };
        write!(f, "({},{},{:.2},{})", self.source, self.sink, self.strength, active)
    }
}

#[derive(Copy, Clone)]
struct Gene {
    dna: u32,
    active: bool,
    source: Node, 
    sink: Node,
    strength: f32,
}

impl Gene {
	fn new() -> Self {
		let mut rng = rand::thread_rng();	
		let dna = rng.gen();
		Gene::with_dna( dna )
	}

	fn with_dna(dna :u32) -> Self {
		Gene {
			dna: dna,
			active: false, // inactive on creation 
			source: Gene::get_source(dna),
			sink: Gene::get_sink(dna),
			strength: Gene::get_strength(dna),
		}
	}	
	
	// returns a mutated copy of self
	fn mutate(&self) -> Self {
		let mut rng = rand::thread_rng();
		let b1 = rng.gen_range(0..32);
		let b2 = rng.gen_range(0..32);

		// let's swap two bits 
		let mut dna = self.dna;
		dna = dna ^ (1 << b1);
		dna = dna ^ (1 << b2);
		
		Gene::with_dna(dna) // just return new one for now...
	}

	fn get_strength(dna :u32) -> f32 {
		let s = (dna & 0xffff) as i16;
		let div = (0xffffu16 >> 1) as i16;
		(s as f32) / (div as f32) * Config::get().strength_mult
	}

	fn get_source(dna :u32) -> Node {
		let byte = ((dna >> 24) & 0xff) as u8; // source in the first byte
		let node_type = NodeType::INPUT;
		Gene::get_node( byte, node_type )
	}

	fn get_sink(dna :u32) -> Node {
		let byte = ((dna >> 16) & 0xff) as u8; // sink in the second byte
		let node_type = NodeType::OUTPUT;
		Gene::get_node( byte, node_type )
	}

	fn get_node(byte :u8, mut node_type :NodeType) -> Node {
		if 0x80 == (byte & 0x80) { // switch node type if first bit set
			node_type = NodeType::HIDDEN;
		}
		let index = (byte & 0b01111111) as usize; // mask out first bit
		let idx = match node_type {
			NodeType::INPUT => index % Config::get().inputs.len(),
			NodeType::HIDDEN => Config::get().inputs.len() + (index % Config::get().neurons),
			NodeType::OUTPUT => Config::get().inputs.len() + Config::get().neurons + (index % Config::get().outputs.len()),
		};
		Node { idx:idx, node_type:node_type }
	}
}

impl Gene { // not sure where to put this function. Auxilary really. 
	fn set_active_genes(genes :&mut Vec<Gene>) { // genome:&mut [Gene] ) {
		//const NODE_COUNT:usize = 24;
		//let mut sourcing = [false; NODE_COUNT];
		//let mut sinking = [false; NODE_COUNT]; // we could just copy sourcing... but how to copy array?

		let mut sourcing = vec![false; Config::get().node_count()]; // create dense vectors 
		let mut sinking = vec![false; Config::get().node_count()]; // to help us keep track of nodes that are potentially sourcing or sinking signals.
		let mut hidden_edges = Vec::new();

		//for chro in self.chromos.iter() {
		//	for g in chro.genes.iter() {

		for g in genes.iter() {
			// find all the DIRECT sources. 
			if matches!(g.source.node_type, NodeType::INPUT) {
				sourcing[g.sink.idx] = true;
			}
			// find all the DIRECT sinks. 
			if matches!(g.sink.node_type, NodeType::OUTPUT) {
				sinking[g.source.idx] = true;
			}
			// find lateral hidden nodes. (while we're at it)
			if matches!(g.source.node_type, NodeType::HIDDEN) & 
			   matches!(g.sink.node_type, NodeType::HIDDEN) {
				hidden_edges.push( g );
			}
		}

		// Enable sources for hidden layer. (I'm sure there's a better way to do all this... probably like a recursive one-liner for this whole functions, but whatevs G)
		'outer1: loop { 
			for g in hidden_edges.iter() {
				if sourcing[g.source.idx] & !sourcing[g.sink.idx] {
					sourcing[g.sink.idx] = true;
					// println!("Source found! {}", g);
					continue 'outer1; // Go back to beginning in case one of those previous edges are now valid.
				}
			} // if we made it through all edges without adding any, time to break.
			break; // do we really need the break?? lol
		}

		// Enable sinks for hidden layer. 
		'outer2: loop { 
			for g in &hidden_edges {
				if sinking[g.sink.idx] & !sinking[g.source.idx] {
					sinking[g.source.idx] = true;
					// println!("Sink found! {}", g);
					continue 'outer2;
				}
			} // if we made it through all edges without adding any, time to break.
			break;
		}

		// Now that we have all sinking and sourcing genes, we can use a little logic, 
		// To activate all genes connected to a source or sink.
		//for chro in self.chromos.iter_mut() {
		//	for g in chro.genes.iter_mut() {
		for g in genes.iter_mut() { 
			g.active = false;
			if matches!(g.source.node_type, NodeType::INPUT) & 
			   matches!(g.sink.node_type, NodeType::OUTPUT) {
				g.active = true;
			} else
			if matches!(g.source.node_type, NodeType::INPUT) & 
			   matches!(g.sink.node_type, NodeType::HIDDEN) {
				if sinking[g.sink.idx] {
					g.active = true;
				}
			} else
			if matches!(g.source.node_type, NodeType::HIDDEN) & 
			   matches!(g.sink.node_type, NodeType::HIDDEN) {
				if sourcing[g.source.idx] & sinking[g.sink.idx] {
					g.active = true;
				}
			} else 
			if matches!(g.source.node_type, NodeType::HIDDEN) & 
			   matches!(g.sink.node_type, NodeType::OUTPUT) {
				if sourcing[g.source.idx] {
					g.active = true;
				}
			} else {
				println!("Something bizarre happening: {}", g);
			}
		}
	}
}


struct Chromo {
	genes :Vec<Gene>, // contains info for working 
}

impl Chromo {
	fn init() -> Self {
		let mut genes = Vec::new();
		genes.push( Gene::new() ); // we'll always have at least one gene
		Self {
			genes: genes,
		}
	}

	fn new() -> Self {
		let mut chromo = Chromo::init();
		while !chromo.is_complete() {
			chromo.genes.push( Gene::new() );
		}
		return chromo
	}

	// return a mutate copy of ourself
	fn mutate(&self) -> Self { 
		// Let's have one mutation per chromosome (for now)
        let mut rng = rand::thread_rng();
		let chance = 1. / (self.genes.len() as f32); // chance for any gene to mutate
		
		let mut genes = Vec::new();
		for gene in self.genes.iter() {
			if chance >= rng.gen_range(0.0..1.0) {
				genes.push( gene.mutate() );
			} else {
				genes.push( gene.clone() );
			}
		}
		let mut chromo = Chromo { genes };
		while !chromo.is_complete() {
			chromo.genes.push( Gene::new() );
		}
		return chromo
	}

	fn is_complete(&mut self) -> bool {
		if Config::get().use_chromo {
			self.set_active();
			self.is_active() // make sure chromosome is a functional network.
		} else {
			self.genes.len() >= 1
		}
	}

	fn set_active(&mut self) { // -> Bool (we could get rid of these two functions)
		Gene::set_active_genes( &mut self.genes );
	}

	fn is_active(&self) -> bool {
		for g in self.genes.iter() {
			if g.active {
				return true;
			}
		}
		return false
	}
}

struct Brain {
	node_state: Vec<f32>,
	inputs: Vec<Node>,
	nurons: Vec<Nuron>,
	outputs: Vec<Nuron>,
}

impl Brain { 
	fn new( genome :&Genome ) -> Self {
		// Find active neurons.
		let mut nurons: Vec<Nuron> = Vec::new();
		'find_nuron: for chro in genome.chromos.iter() {
		  for gene in chro.genes.iter() {
			// let's only look at sources (arbitrary, could have selective sinks) 
			if gene.source.is_hidden() & gene.active {
				// Ignore if it's already been added to nurons
				for nuron in nurons.iter() { // vs just "... in nurons"? 
					if nuron.node == gene.source {
						continue 'find_nuron;
					}
				} 
				
				// Find all incoming nodes
				let mut incoming: Vec<Gene> = Vec::new();
				for c2 in genome.chromos.iter() {
				  for g2 in c2.genes.iter() {
					if g2.active & (g2.sink == gene.source) {
						incoming.push( *g2 );
					}
				  }
				}
				
				assert!( incoming.len() > 0 );
				nurons.push( Nuron { node:gene.source, incoming:incoming, state:0f32 });
			}
		  }
		}

		// Find active inputs 
		let mut inputs :Vec<Node> = Vec::new();
		for chro in genome.chromos.iter() { // use iter, because sometimes you don't know/care-to-look-up if the current vector is & or not. More robust. :) 
		  for gene in chro.genes.iter() {
			if gene.source.is_input() & gene.active {
				if !inputs.contains( &gene.source ) {
					inputs.push( gene.source );
				}
			}
		  }
		}
		
		// Setup active outputs 
		let mut outputs :Vec<Nuron> = Vec::new();
		'find_output: for chro in genome.chromos.iter() {
		  for gene in chro.genes.iter() {
			if gene.sink.is_output() & gene.active {
				let node = gene.sink; // & is not necessary here because Node is copyable, right? 
				
				// Ignore if it's already been added to outputs
				for output in outputs.iter() { // vs just "... in nurons"? 
					if output.node == node {
						continue 'find_output;
					}
				} 
				
				// Find all incoming nodes
				let mut incoming: Vec<Gene> = Vec::new();
				for c2 in genome.chromos.iter() {
				  for g2 in c2.genes.iter() {
					if g2.active & (g2.sink == node) {
						incoming.push( *g2 );
					}
				  }
				}

				assert!( incoming.len() > 0 );
				outputs.push( Nuron { node:node, incoming:incoming, state:0f32 });
			}
		  }
		}

		Self { // let brain =
			inputs, 
			nurons, 
			outputs, 
			node_state: vec![0.; Config::get().node_count()],
		}
		// brain.print(); 
		// return brain
	}

	#[allow(dead_code)]
	fn print(&self) {
		println!("Inputs: {:?}", &self.inputs);		
		print_nurons( "Neurons", &self.nurons );
		print_nurons( "Outputs", &self.outputs );		
		fn print_nurons( title: &str, nurons: &Vec<Nuron> ) {
			// for the life of me, can't figure out how to print the vector of Nurons, using the above Display fn
			// Problem is, when I derive Debug for Nuron, it doesn't want to use the Display fn, maybe because it's not debug info?
			// So in this case, we probably have to derive a "debug" display clause, but it doesn't make sense to me, why they should be different. 
			// See this is the problem with Rust, ... just things like this (printing vec, of struct object with custom Display), are like, just not possible? And that's okay? Why? Who made this decision? I don't think I'm okay with it. :/ 
			print!("{}:", title);
			for nuron in nurons.iter() { print!("{}", nuron); }
			println!(" END");
		}
	}	

	fn get_inputs<T:Creature>( &mut self, t :&T ) {
		for node in self.inputs.iter() {
			self.node_state[ node.idx ] = t.calc_input( &node.get_name() );
		}
	}

    fn process_inputs( &mut self ) {
		// first sum all nodes in hidden and output layer, using node_state from last live() iteration, plus with updated inputs of course
		for nuron in self.nurons.iter_mut().chain( self.outputs.iter_mut() ) {
			let mut state:f32 = 0f32;
			for gene in nuron.incoming.iter() {
				let s:f32 = gene.strength;
				let value = self.node_state[ gene.source.idx ];
				state += value * s;
			}
        	nuron.state = state;
		}

		// now write the values to the node_state
		for nuron in self.nurons.iter() {
			self.node_state[ nuron.node.idx ] = math::tanh( nuron.state ); // squash them down!
		}
		
		//// not really necessary... but we're doing it for completeness, maybe someone will expect it to be there in the future. :|
		//for nuron in outputs.iter() {
		//	node_state[ nuron.root.idx() ] = nuron.state; // don't squash outputs. We'll manually 
		//}
    }
}

pub struct Nuron { // don't want to mispell in my code, so keep it phoenetic :P
    pub state: f32, // current value
    pub node: Node,
    incoming: Vec<Gene>,
}

impl std::fmt::Display for Nuron {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = format!(" {}:", self.node);
        for gene in &self.incoming { // so v.iter() and &v are the same thing! 
            output += &format!("({},{:.1}),", gene.source, gene.strength );
        }
        write!(f, "{}", output)
    }
}

#[derive(Clone)]
pub struct Config {
	pub population :usize,
	pub lifespan: usize,
	pub genome_size :usize, 
	pub use_chromo :bool,
	pub independent :bool,
	pub strength_mult :f32, // multiplier for gene strengths
	// pub nodes: (usize, usize, usize), // (inputs, nurons, outputs)
	pub inputs :Vec<String>,
	pub outputs :Vec<String>,
	pub neurons :usize,
}

impl Config {
	fn avg_life(&self) -> usize { self.lifespan }

	pub fn set( cfg :Self ) {
	 	{
	 		let mut guard = CONFIG.lock().unwrap();
	 		*guard = cfg; // this actually work?! Ugh, this is ugly. Whatever. 
	 	} // I think a block is need to release the lock? (Lolol)
	}

	// Something just seems wrong here... whatever. I hate this BS myself. But what can you do other than pass around config everywhere?? (which I'm not going to do.)
	fn get() -> Config { 
		(*CONFIG.lock().unwrap()).clone()
	}

	// do I need a lifetime here??? Ughhhh!!!!
	fn node_name(&self, node :&Node) -> String {
		if node.idx < self.inputs.len() {
			format!("{}", &self.inputs[ node.idx ])
		} else if node.idx >= (self.inputs.len() + self.neurons) {
			assert!( node.idx < (self.inputs.len() + self.neurons + self.outputs.len()) );
			format!("{}", &self.outputs[ node.idx - (self.neurons + self.inputs.len()) ])
		} else {
			String::from("??")
		}
	}

	fn node_count(&self) -> usize {
		self.inputs.len() + self.neurons + self.outputs.len()
	}
}

impl Genome {
	fn new() -> Self {
		let mut chromos = Vec::new();
		for _i in 0..Config::get().genome_size {
			chromos.push( Chromo::new() );
		}
		Genome::with_chromos( chromos )
	}
		
	fn with_chromos( chromos :Vec<Chromo> ) -> Self {
		let mut genome = Self {
			offspring: 0, // prevent division by zero? (shrugs)
			max_fitness: 0., // prevent division by zero? (shrugs)
			alive: true,
			age: 0,
			chromos: chromos,
		};

		genome.set_active_genes(); // Find any more genes that may be actived from combined chromosome networks
		// do we really need to be doing this here, or when we build the brains out?
		// Just do it. Building a genome always finds/sets the active genes. Sounds like a good rule to me.
		
		return genome
	}

	// Asexual reproduction method, creates a mutates clone
	fn bud(&mut self) -> Self {
		self.offspring += 1;
		// First let's build the chromosomes

		let mut chromos = Vec::new();
		for chro in self.chromos.iter() {
			chromos.push( chro.mutate() );
		}
		Genome::with_chromos( chromos )
	}
	
	fn set_active_genes(&mut self) {
		// We need to pull all the genes together. 
		let mut genes = Vec::new();
		for chro in self.chromos.iter_mut() {
			for g in chro.genes.iter() {
				genes.push( *g )
			}
		}
		Gene::set_active_genes( &mut genes );
	}
	
	#[allow(dead_code)]
    fn print_genes(&self) {
		print!("Genome: ");
        for chro in self.chromos.iter() {
			print!("Chromo: ");
			for g in chro.genes.iter() {
				print!("{}, ", g);
			}
		}
		println!("END");
    }
}

pub struct Genome {
	offspring: usize,
	max_fitness: f32,
	pub alive: bool,
	pub age: usize,
	chromos :Vec<Chromo>,
}

pub struct World<T:Creature> {
	brains :Vec<Brain>, // 3 parallel arrays: brains, genomes, creatures
	pub genomes :Vec<Genome>,
	pub creatures :Vec<T>,
	fertile: Vec<usize>, // list all creatures to reproduce
}

impl<T:Creature> World<T> {
	pub fn new() -> Self {
		let mut genomes = Vec::new();
		let mut brains = Vec::new();
		let mut creatures = Vec::new();
		for i in 0..Config::get().population {
			genomes.push( Genome::new() );
			brains.push( Brain::new( &genomes[i] ) );
			creatures.push( T::init() );
		}
		Self { genomes, brains, creatures, fertile: Vec::new() }
	}

	pub fn birth(&mut self, genome :Genome) {
		let mut reclaim = -1;
		for g_id in 0..self.genomes.len() {
			if !self.genomes[g_id].alive {
				reclaim = g_id as isize;
				break;
			}
		}
		
		if reclaim >= 0 {
			let id = reclaim as usize;
			self.brains[id] = Brain::new( &genome ); // gotta add brain first; Clean brain
			self.creatures[id] = T::init(); // and a new body while we're at it
			self.genomes[id] = genome; // and then genomes takes ownership
		} else { // or else add to the back. 
			self.brains.push( Brain::new( &genome ) ); // gotta add brain first
			self.genomes.push( genome );
			self.creatures.push( T::init() );
		}
	}

	// live carries out processes for all creatures
	pub fn live(&mut self) { 
		self.advance( Config::get().avg_life() );
	}

	pub fn advance(&mut self, total_steps :usize) {
		// Independent lives will reproduce at the end of their lives (or step sequences, if you will), for the sake of computational loops and efficiency. We need handle their cycles a little differently.
		if Config::get().independent {
			let avg = Config::get().avg_life();
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
			for id in 0 .. self.genomes.len() { // iter().enumerate() {
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

	fn offspring_needed(&self, steps :&usize) -> usize {
		let mut needed = (steps * Config::get().population ) as f32 / (Config::get().avg_life() as f32);
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
		for genome in self.genomes.iter() {
			if !genome.alive { continue; }
			max = f32::max(genome.max_fitness, max);
		};
		return max
	}
		
	fn sum_fitness(&self) -> f32 {
		let mut tot = 0.;
		for genome in self.genomes.iter() {
			if !genome.alive { continue; }
			tot += f32::abs(genome.max_fitness); // avoid negative fitness possibiities
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
			for (id, genome) in self.genomes.iter().enumerate() {
				if !genome.alive { continue; }
				tot += f32::abs(genome.max_fitness);
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
			let g_id = self.fertile.pop().unwrap();
			let genome = self.genomes[g_id].bud();
			self.birth( genome );
		}

		// Maybe wait until we have a certain pool size of them, and then they can reproduce. 
		// Or perhaps they become "fertile". 
		// And then we look for fertile partners. 
		// So during it's during test_fitness, which turns on their fertility? 
		// In this way, if something is picked twice, then I suppose it's fertility score goes up?
		// And then when it finds a chance to reproduce, 
		// It does so at the next opportunity. 
		// But perhaps it should wait for a compatible partner? 
		// Ugh, am I overthinking this? Does any body care? 
		// Okay, we pick the winners, and now they need to reproduce. 
		// Maybe people want to change the way winners are picked. There could be a default method. 
		// Or the user could provide their own. Why not? 
		
		// Just print it out. 
		// println!( "Winners: {:?}", &self.fertile );

	}

	// cannot borrow self as mutable more than one time. 
	fn independent_steps( &mut self, g_id :usize, steps :&usize ) {
	// This genome/creature/brain bullshit seems to be gunking things up. 
	// Do I really want fine control over mutability of each separately?
	// It seems it's making more problems than it's worth!

		let genome = &mut self.genomes[g_id];
		if !genome.alive { return (); } // sanity check
		let brain = &mut self.brains[g_id];
		let creature = &mut self.creatures[g_id];

		'step_loop: for _s in 0..*steps {
			brain.get_inputs( creature );
			brain.process_inputs();
			creature.apply_outputs( &brain.outputs );

			// Won't allow this! Because I borrow against self too many times. 
			// So much for the reusability of the age_genome function. 
			//	for _s in 0..*steps {
			//		self.age_genome( g_id.clone(), 1 );
			//	}
			
			genome.max_fitness = f32::max( genome.max_fitness, creature.fitness(genome) );

			genome.age += 1;
			if creature.to_die( genome ) {
				genome.alive = false;
				break 'step_loop;
			}

		}

	}
	
	// Efficiency concern: If objects are independent of each other in the environment, 
	// We can compute all steps at once for one creature, in memory, without fear of thrashing/swapping between the potential 1000's of creatures in our environment for each step. 
	// more: [docs/independence-efficiency.txt]

	fn step(&mut self) {
		// first collect the inputs
        for (id, genome) in self.genomes.iter().enumerate() {
			if !genome.alive { continue; }
			let brain = &mut self.brains[id]; // wait, is self.brains[id] below moving ownership?
			brain.get_inputs( &self.creatures[id] );
		}

		// now compute the outputs (and do stuff)
		for (id, genome) in self.genomes.iter_mut().enumerate() {
			if !genome.alive { continue; }

			let brain = &mut self.brains[id];  
			brain.process_inputs(); // also squashes inner nodes
			self.creatures[id].apply_outputs( &brain.outputs );

			// and update max_fitness while we're here: 
			genome.max_fitness = f32::max( genome.max_fitness, self.creatures[id].fitness(genome) );
        }

		// And finally time to die
		for id in 0 .. self.genomes.len() { // self.genomes.iter_mut().enumerate() {
			self.age_genome( id, 1 ); // handles dieing as well as aging. 
		}
		// self.expunge_dead(); //
	}
	
	// ** Note, this function is copied in the independent_steps() method!!
	//  , so be sure to paste changes there if applicable!
	fn age_genome( &mut self, g_id :usize, steps :usize) {
		let genome = &mut self.genomes[g_id];
		genome.age += steps; // We're all a little bit older (sure, even the ghosts can age)
		if self.creatures[g_id].to_die( genome ) {
			genome.alive = false;
		}
	}

	// no more expunge dead is necessary. We are reclaiming the bodies. It's inefficient anyway. 
	// But we need to move this disuccssion to the "finding new index" function for new creatures
	// Okay sure, when they insert into the vector. If there are any dead ones at the front, but it there. 
	// Otherwise create a new one. What's the harm? There will eventually be one in the front. 
	// This way, we keep our dead around until it gets re-incarnated. O(1) time. Lol. 
//	fn expunge_dead(&mut self) {
//		'check_front: for (id, genome) in self.genomes.iter().enumerate() {
//			if !self.genome.alive { // we'll pop the dead ones off the front (of the 3 parallel vectors)
//				// really we should stash it somewhere. Like in a "dead" list. 
//				// ultimately, we want to see the peak output. Right? ... [tbd: docs/deliverables.txt]
//				// regardless, new process: The body gets reclaimed 
//
//				if genome.age > 2*Config::get().lifespan {
//					// Do we need a mutex here to make sure these things need to be done together? 
//					// ... see, this is why I think creature (and brain) should be in a pub Genome struct. 
//					//{ // still might want a mutex for accessing the genome... ??? That is far off though, needing that feature, and implementing. Lol. 
//						self.genomes.erase( self.genomes.begin() );
//						self.brains.erase( self.brains.begin() );
//						self.creatures.erase( self.creatures.begin() );
//					//}
//				}
//
//			} else { // until we find an alive one. 
//				break 'check_front; // and remove the rest later when it's more convenient to do so
//			}
//		}
//	}

	pub fn evolve(&mut self) {
		// for all the genes, we need some kind of function
		// gworld. ... 
		// Something produced during the "live" function need to be evaluated.
		// How to do this? 
		// do we just do it all in the "live" step? 
		// Okay, this is when the "going gets tough."
		// Ha! It's like i have 10x more comments than actual code. LOLOL

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

pub trait Creature {
	fn init() -> Self;
	fn calc_input(&self, input :&str) -> f32;
	fn apply_output(&self, output :&str);
	fn apply_outputs(&mut self, outputs :&Vec<Nuron>);
	fn to_die(&self, genome :&Genome) -> bool { 
		genome.age > Config::get().lifespan
		// ultiamtely, might want it some probabalistic based on a continuous fit function. 
	}
	fn fitness(&self, genome :&Genome) -> f32; 

}

// Also, gotta give a shout out to genevo. It looks to be an excellent library. I was really considering using it, and I still feel like I _should_ be building upon it. 
// However a couple points:
// 1) I'm adding this concept of chromosome, and plan to explore diploid/haploid etc, types of breedings, among other things
// 2) So it seems like it'll be easier to tinker if I just do this from the ground up as opposed to trying to fit genevo's model.
// 3) I just want to learn Rust, and the reasons behind generics and traits, and how to create libraries, and to contribute my code.
// 
// I hope you like it. :)
// (And no I won't use rustfmt just yet. I actually _like_ the current formatting choices. :/ )
