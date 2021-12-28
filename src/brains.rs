use super::node::Node;
use super::genes::{Gene, Genome};
use super::config::Config;

pub(crate) struct Nuron { // don't want to mispell in my code, so keep it phoenetic :P
    pub(crate) state: f32, // current value
    pub(crate) node: Node,
    pub(crate) incoming: Vec<Gene>,
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

pub(crate) struct Brain {
	pub(crate) node_state: Vec<f32>,
	pub(crate) inputs: Vec<Node>,
	pub(crate) nurons: Vec<Nuron>,
	pub(crate) outputs: Vec<Nuron>,
}

impl Brain { 
	pub(crate) fn new( genome :&Genome ) -> Self {
		// Find active neurons.
		let mut nurons: Vec<Nuron> = Vec::new();
		'find_nuron: for chro in genome.chromos.iter() {
		  for gene in chro.genes.iter() {
			// let's only look at sources (arbitrary, could have selective sinks) 
			if gene.source.is_hidden() & gene.active {
				// Ignore if it's already been added to nurons
				for nuron in nurons.iter() { 
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
				for output in outputs.iter() { 
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
			print!("{}:", title);
			for nuron in nurons.iter() { print!("{}", nuron); }
			println!(" END");
		}
	}	

}
