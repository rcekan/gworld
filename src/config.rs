use super::node::Node;

#[derive(Clone)]
pub struct Config {
	pub verbose :String, // "low", "high", or _ ;  could use enum... but easier for user this way.
	pub population :usize,
	pub lifespan: usize,
	pub genome_size :usize, 
	pub use_chromo :bool, // [See: docs/chromos.txt]
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

	pub(crate) fn log(s :&str) -> bool {
		Config::get().verbosity( s )
	}

	fn verbosity(&self, s :&str) -> bool {
		let v = self.verbose.to_lowercase();
        match s.to_lowercase().as_str() {
 			"on" => 
 				match v.as_str() {
 					"silent" => false,
 					_ => true,
 				},
             "low" => // both low and high qualify
 				match v.as_str() {
 					"low" => true,
 					"high" => true,
 					_ => false,
 				},
 			"high" => // only high applies
 				match v.as_str() {
 					"high" => true,
 					_ => false,
 				},
 			_ => false,
 		}
	}

}

// Got sick of passing around the config everywhere. Let's try to make it a global variable. 
use lazy_static::lazy_static;
use std::sync::{Mutex}; // , MutexGuard};

lazy_static!{
    static ref CONFIG :Mutex<Config> = Mutex::new( Config { 
		// these will all get wiped out anyway. It's so silly. 
		// There must be a way to pass only the arguments that you need to apply. 
		// Would probably have to be a HashMap, where keys are struct field names. 
		// But Rust probably doesn't allow this. Can you access and set field values based on field name string, like js or python?
		verbose: "low".to_string(),
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

	// so here's kind of an issue...
	// I want to be able to pass in only the config variables I care about
	// Also want to "automagically" calculate other variables, like nodes
	// Maybe a hashmap then? Where the values get copied over ... cleverly? Does rust even allow that? Probably not without unsafe. 
	// Let's just spell it all out for now. 

