# gworld
Rust library for genetic algorithms

## Usage notes
- [world.rs](./src/world.rs) defines the primary objects/traits. 

- `World` contains 1 environment and many creatures, which you will define via the `Environs` and `Creature` traits. 

- When setting the config, a "functional" chromosome is defined as a collection of genes that describe a full path from input to output. It is coupled with mutation rate at the moment, and keeping `use_chromo` set to `true` will reduce mutation rate. 

## Example

It doesn't do much, but it evolves. I will post a more complex example soon hopefully.

[examples/blobs.rs](./examples/blobs.rs)

```rust
use gworld::{math, World, Config, Environs, Creature};

fn main() {
	Config::set( Config {
		inputs: ["X", "Y"].iter().map(|&s| s.into()).collect(),
		outputs: ["MOVX", "MOVY"].iter().map(|&s| s.into()).collect(),
		neurons: 3,
		strength_mult: 4.0, // multiplier for gene strengths
		population: 50, 
		lifespan: 100, 
		genome_size: 6, // number of chromosomes
		use_chromo: true, // multiple genes per functional chromosome?
		independent: false, // do the creatures (not) interact with each other?
		verbose: "none".to_string(), // options: silent/low/high
	});

	let mut world :World<MyEnv, Blob> = World::new(); 
	world.live(); // will advance the world #lifespan steps 
	world.advance( 1000 ); // will advance the world 1000 steps
	
	// world.environs to access MyEnv structure
	// world.organisms[i].creature to access Blob creatures
}

struct MyEnv {}

impl Environs for MyEnv {
	type Creature = Blob;
	fn new() -> Self { Self{} }
}

struct Blob {
	x: f32,
	y: f32,
}

impl Creature for Blob {
	type Env = MyEnv;
	
	fn new( _env: &mut Self::Env ) -> Self {
		Self { // may want to generate x, y from env data
			x: 10.,
			y: 10.,
		}
	}
	
	fn act( &mut self, _env: &mut Self::Env ) -> f32 {
		return 0. // need to return a fitness value
	}
	
	fn rx_input( &self, input: &str, _env: &Self::Env ) -> f32 {
		match input {
			"X" => self.x,
			"Y" => self.y,
			_ => { 
				println!("rx_input: no match found for: {}", input );
				return 0.
			},
		}
	}
	
	fn tx_output( &mut self, output: &str, value: f32, _env: &Self::Env ) {
		match output { // may want to check with env and make sure this is a valid location to move to!
			"MOVX" => self.x = math::tanh( value ),
			"MOVY" => self.y = math::tanh( value ),
			_ => println!("tx_output: no match found for: {}", output ),
		}
	}
}
```

## Future work

The breeding is currently super basic. Asexual, and one bit-swap mutation per chromosome. 

Ultimately, the idea is to use chromosomes to enhance breeding, perhaps explore diploid (tri, n-ploid) mating strategies. 

The goal is for the library to take out all the boilerplate work when setting up a genetic algorithm. 

I've currently used it to create a painting algorithm, and I plan to reproduce an example that mimics the work done in this video:
[I programmed some creatures. They Evolved.](https://www.youtube.com/watch?v=N3tRFayqVtk&t=1392s)

I also had this creation in mind, when creating the code. I can't say for sure whether gwould could be used to create something like this, but I think it could get close, and hopefully will evolve to have the capability. 
[How I created an evolving neural network ecosystem](https://www.youtube.com/watch?v=myJ7YOZGkv0)

The whole idea is that the `act` method mutates the environment. That is likely where the meat and bones of your creature behaviors will go. 

Ultimately, it might be interesting to have multiple species evolving, perhaps with different fit functions. More on this in [docs/multiple-species.txt](./docs/multi-species.txt).

So much to do. So little time. I'll continue using it for personal projects and add to it as needed. 

If you're using the library and have a feature request and/or would like to contribute, I'd love to hear from you!
(I guess notify me `@rcekan` with an issue, is that the only way?)
