use super::Node;
use super::CONFIG;

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
	pub fn set( cfg :Self ) {
	 	{
	 		let mut guard = CONFIG.lock().unwrap();
	 		*guard = cfg; // this actually work?! Ugh, this is ugly. Whatever. 
	 	} // I think a block is need to release the lock? (Lolol)
	}

	// Something just seems wrong here... whatever. I hate this BS myself. But what can you do other than pass around config everywhere?? (which I'm not going to do.)
	pub(crate) fn get() -> Config { 
		(*CONFIG.lock().unwrap()).clone()
	}

	pub(crate) fn node_count(&self) -> usize {
		self.inputs.len() + self.neurons + self.outputs.len()
	}

	pub(crate) fn node_name(&self, node :&Node) -> String {
		if node.idx < self.inputs.len() {
			format!("{}", &self.inputs[ node.idx ])
		} else if node.idx >= (self.inputs.len() + self.neurons) {
			assert!( node.idx < (self.inputs.len() + self.neurons + self.outputs.len()) );
			format!("{}", &self.outputs[ node.idx - (self.neurons + self.inputs.len()) ])
		} else {
			String::from("??")
		}
	}
}

