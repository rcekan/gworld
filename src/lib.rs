// A library providing genome algorithms. 
// Note: This library makes more sense when read from the bottom up. It's just how I roll. Sorry not sorry.
pub mod math;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub struct Nuron { // don't want to mispell in my code, so keep it phoenetic :P
    pub state: f32, // current value
    pub root: Node,
    incoming: Vec<Gene>,
}

impl Nuron {
    fn sum_inputs( &mut self, node_state:&[f32] ) {
        let mut state:f32 = 0f32;
        for gene in self.incoming.iter() {
            let s:f32 = gene.strength;
            let value = node_state[ gene.source.idx ];
            state += value * s
        }
        self.state = state;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum NodeType {
    INPUT,
    HIDDEN,
    OUTPUT,
}

#[derive(Copy, Clone)]
pub struct Node {
    pub idx: usize, // unique across all nodes (TODO: ... although, there's some consideration to be made for the fact that the inner neurons don't necessarily need to be a particular inner nueron, possibly we can "shift" the inner neurons as a mutation? Or we can manually shift the index for them such that various chromosomes don't overlap functions with each other's inner neurons. Not sure that's what we'd want, but it could be interesting to consider this "independence" of the inner neurons, by adjusting their assigned "idx" values algorithmically to "make sense" for a different model/constrant/design.)
    pub node_type: NodeType,
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
//    fn empty() -> Node { // create an empty node.
//        Node { 
//            idx: 0xFFusize, // FF special case, 7F is the highest possible number anyhow (ie: 7bit representation in dna)
//            node_type: NodeType::HIDDEN,
//        }
//    }
    pub fn get_name(&self) -> &str {
		// World::config().node_names[ self.idx ]
		"??"
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
	    use rand::Rng;
		let mut rng = rand::thread_rng();	
		let dna = rng.gen();
		Gene {
			dna: dna,
			active: false, // inactive on creation 
			source: Gene::get_source(dna),
			sink: Gene::get_sink(dna),
			strength: Gene::get_strength(dna),
		}
	}

	fn get_strength(dna :u32) -> f32 {
		let s = (dna & 0xffff) as i16;
		let div = (0xffffu16 >> 1) as i16;
		(s as f32) / (div as f32) * World::config().strength_mult
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
			NodeType::INPUT => index % World::config().inputs.len(),
			NodeType::HIDDEN => World::config().inputs.len() + (index % World::config().neurons),
			NodeType::OUTPUT => World::config().inputs.len() + World::config().neurons + (index % World::config().outputs.len()),
		};
		Node { idx:idx, node_type:node_type }
	}
}

impl Gene { // not sure where to put this function. Auxilary really. 
	fn set_active_genes(genes :&mut Vec<Gene>) { // genome:&mut [Gene] ) {
		//const NODE_COUNT:usize = 24;
		//let mut sourcing = [false; NODE_COUNT];
		//let mut sinking = [false; NODE_COUNT]; // we could just copy sourcing... but how to copy array?

		let mut sourcing = vec![false; World::config().node_count()]; // create dense vectors 
		let mut sinking = vec![false; World::config().node_count()]; // to help us keep track of nodes that are potentially sourcing or sinking signals.
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
			break;
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
			chromo.set_active(); 
		}
		return chromo
	}

	fn set_active(&mut self) {
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
	
	fn is_complete(&self) -> bool {
		if World::config().use_chromo {
			self.is_active() // make sure chromosome is a functional network.
		} else {
			self.genes.len() >= 1
		}
	}
}

struct Genome {
	chromos :Vec<Chromo>,
}

// pub struct World <'a, T:GWorld>{
pub struct World {
	genomes :Vec<Genome>,
	// gworld :&'a T,
}

#[derive(Clone)]
pub struct Config {
	pub population :usize,
	pub genome_size :usize, 
	pub use_chromo :bool,
	pub strength_mult :f32, // multiplier for gene strengths
	// pub nodes: (usize, usize, usize), // (inputs, nurons, outputs)
	pub inputs :Vec<String>,
	pub outputs :Vec<String>,
	pub neurons :usize,
}

impl Config {
	fn node_count(&self) -> usize {
		self.inputs.len() + self.neurons + self.outputs.len()
	}
}

impl Genome {
	fn new() -> Self {
		let mut chromos = Vec::new();
		for _i in 0..World::config().genome_size {
			chromos.push( Chromo::new() );
		}
		
		let mut genome = Self {
			chromos: chromos,
		};

		// if World::config().use_chromo {
		genome.set_active_genes(); // Set no matter what. 
		// Honestly, probably while we're at it, could build the nuron network. 
		// Problem is, I'm not sure where I should be doing the "things" now. 
		// Like when/where should I build the nuron net? 
		// Similarly, should I put all the "other" stuff inside gene? Apart from the dna? Isn't it all redundant? 
		// I just feel like generally, I could be organizing my structs and implemenation better. 
		
		return genome
	}

	fn set_active_genes(&mut self) {
		// We need to pull all the genes together. 
		self.print_genes();
		let mut genes = Vec::new();
		for chro in self.chromos.iter_mut() {
			for g in chro.genes.iter() {
				genes.push( *g )
			}
		}
		Gene::set_active_genes( &mut genes );

		// is this really doing what we want? Or is it moving/destroying all genes in the process?
		// Guess we need to check. // Just need to print out before and after. 	
		self.print_genes();
	}

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


impl World {

	// live carries out processes for all the genomes
	pub fn live(&self, mut gworld :&impl GWorld) { 
        for genome in self.genomes.iter() {
			// self.gworld.calc_inputs() // etc

            // let mut canvas = Canvas::init( &start );
            // paint( &genome, &mut canvas );
            // if true { // self.printing { 
            //     canvas.print(); 
            // }
            // canvases.push( canvas )
        }
	}

	pub fn evolve(&mut self, mut gworld :&impl GWorld) {
		// for all the genes, we need some kind of function
		// gworld. ... 
		// Something produced during the "live" function need to be evaluated.
		// How to do this? 
		// do we just do it all in the "live" step? 
		// Okay, this is when the "going gets tough."
		// Ha! It's like i have 10x more comments than actual code. LOLOL

	}

	pub fn new(cfg :Config) -> Self {
		// unsafe { // lol, welcome guests. Trust me, this is the only unsafe line in the whole library! Rust should fix this!!! The only other way is for me to pass config around everywhere, which I was doing before, and it was a nuisance. There needs to be a better way to cover this case. So unsafe it is, until Rust decides that there's absolutely nothing unsafe about allowing a mutable global variable, if you make sure it's always assigned with a properly typed items. Like Really??? And don't give me "but the thread safety blah blah blah" crap. Is my overly pedantic graduate advisor on the Rust design team or something??? Surely there's a way to figure this out...
		// 	CONFIG = cfg; // first, set the global config (it is used in Genome::new() and everywhere basically)
		// }
		{
			let mut guard = CONFIG.lock().unwrap();
			*guard = cfg; // this actually work?! Ugh, this is ugly. Whatever. 
		} // I think a block is need to release the lock? (Lolol)

		let mut genomes = Vec::new();
		for _i in 0..World::config().population {
			// Let's create a genome
			genomes.push( Genome::new() );
		}
		Self { // and return the genomes
			genomes: genomes, 
			// gworld: gworld, // it does seem a little awkward. Like why do we have to pass this object? Can't the canvas just evolve itself? Like have the canvas just take the config. 
		}
	}

	// Something just seems wrong here... whatever. I hate this BS myself. But what can you do other than pass around config everywhere?? (which I'm not going to do.)
	fn config() -> Config { 
		(*CONFIG.lock().unwrap()).clone()
	}
}

// Got sick of passing around the config everywhere. Let's try to make it a global variable. 
use lazy_static::lazy_static;
use std::sync::{Mutex, MutexGuard};

lazy_static!{
    static ref CONFIG :Mutex<Config> = Mutex::new( Config {
		population: 100,
		genome_size: 50,
		use_chromo: true,
		strength_mult: 4.0, // multiplier for gene strengths
		// nodes: (5, 3, 5),
		inputs: Vec::new(),
		outputs: Vec::new(),
		neurons: 5,
	});
}

pub trait GWorld {
	fn calc_input(&self, input :&Node) -> f32;
	fn apply_outputs(&mut self, outputs :&Vec<Nuron>);
}











// Also, gotta give a shout out to genevo. It looks to be an excellent library. I was really considering using it, and I still feel like I _should_ be building upon it. 
// However a couple points:
// 1) I'm adding this concept of chromosome, and plan to explore diploid/haploid etc, types of breedings, among other things
// 2) So it seems like it'll be easier to tinker if I just do this from the groun up as opposed to trying to fit genevo's model.
// 3) I just want to learn Rust, and the reasons behind generics and traits, and how to create libraries, and to contribute my code.
// 
// I hope you like it. :)
// (And no I won't use rustfmt just yet. I actually _like_ the current formatting choices. :/ )
