use super::{Node, NodeType};
use super::config::Config;
use rand::Rng; 
// mod Node; // can I just use NodeType down below now? 

#[derive(Copy, Clone)]
pub(crate) struct Gene {
    pub(crate) dna: u32,
    pub(crate) active: bool,
    pub(crate) source: Node, 
    pub(crate) sink: Node,
    pub(crate) strength: f32,
}

impl std::fmt::Display for Gene {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let active = if self.active { 'T' } else { 'F' };
        write!(f, "({},{},{:.2},{})", self.source, self.sink, self.strength, active)
    }
}

impl Gene {
	pub(crate) fn new() -> Self {
		let mut rng = rand::thread_rng();	
		let dna = rng.gen();
		Gene::with_dna( dna )
	}

	pub(crate) fn with_dna(dna :u32) -> Self {
		Gene {
			dna: dna,
			active: false, // inactive on creation 
			source: Gene::get_source(dna),
			sink: Gene::get_sink(dna),
			strength: Gene::get_strength(dna),
		}
	}	
	
	// returns a mutated copy of self
	pub(crate) fn mutate(&self) -> Self {
		let mut rng = rand::thread_rng();
		let b1 = rng.gen_range(0..32);
		let b2 = rng.gen_range(0..32);

		// let's swap two bits 
		let mut dna = self.dna;
		dna = dna ^ (1 << b1);
		dna = dna ^ (1 << b2);
		
		Gene::with_dna(dna) // just return new one for now...
	}

	pub(crate) fn get_strength(dna :u32) -> f32 {
		let s = (dna & 0xffff) as i16;
		let div = (0xffffu16 >> 1) as i16;
		(s as f32) / (div as f32) * Config::get().strength_mult
	}

	pub(crate) fn get_source(dna :u32) -> Node {
		let byte = ((dna >> 24) & 0xff) as u8; // source in the first byte
		let node_type = NodeType::INPUT;
		Gene::get_node( byte, node_type )
	}

	pub(crate) fn get_sink(dna :u32) -> Node {
		let byte = ((dna >> 16) & 0xff) as u8; // sink in the second byte
		let node_type = NodeType::OUTPUT;
		Gene::get_node( byte, node_type )
	}

	pub(crate) fn get_node(byte :u8, mut node_type :NodeType) -> Node {
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


pub(crate) struct Chromo {
	pub(crate) genes :Vec<Gene>, // contains info for working 
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

pub(crate) struct Genome {
	pub(crate) chromos :Vec<Chromo>,
}

impl Genome {
	pub fn new() -> Self {
		let mut chromos = Vec::new();
		for _i in 0..Config::get().genome_size {
			chromos.push( Chromo::new() );
		}
		Genome::with_chromos( chromos )
	}
		
	pub fn with_chromos( chromos :Vec<Chromo> ) -> Self {
		let mut genome = Self {
			chromos: chromos,
		};

		genome.set_active_genes(); // Find any more genes that may be actived from combined chromosome networks
		// do we really need to be doing this here, or when we build the brains out?
		// Just do it. Building a genome always finds/sets the active genes. Sounds like a good rule to me.
		
		return genome
	}

	// Asexual reproduction method, creates a mutated clone
	pub fn bud(&self) -> Self {
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
