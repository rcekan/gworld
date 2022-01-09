use std::io::stdin;
use rand::Rng;
use gworld::{math, World, Config, Environs, Creature};

fn main() {
	Config::set( Config {
		// inputs: ["DT", "DB", "DL", "DR"].iter().map(|&s| s.into()).collect(),
		// inputs: ["X", "Y", "DB", "DT"].iter().map(|&s| s.into()).collect(),
		inputs: ["DB", "DT"].iter().map(|&s| s.into()).collect(),
		outputs: ["MOVX", "MOVY"].iter().map(|&s| s.into()).collect(),
		neurons: 2,
		strength_mult: 4.0, // multiplier for gene strengths
		population: 20,
		lifespan: 20,
		genome_size: 4, // number of chromosomes
		use_chromo: true, // multiple genes per functional chromosome?
		independent: false, // do the creatures (not) interact with each other?
		verbose: "silent".to_string(), // options: silent/low/high
	});

	let mut world :World<MyEnv, Blob> = World::new(); 
	world.environs.print();

	let generations = 5;
	for _i in 0..generations {
		for _i in 0..Config::get().lifespan {
			println!("{}", world.fitness_stats());
			stdin().read_line(&mut String::new());
			world.advance(1); // will advance the world #lifespan steps 
			world.environs.print();
		}
	}
}

const DIM :usize = 8; // Dimensions for the map: DIM x DIM
struct MyEnv {
	next_name: usize,
	map: [[(usize, usize); DIM]; DIM], // map will store name of the creature there
}

impl MyEnv {
	fn print(&self) { 
        for row in self.map.iter() {
			print!( "[ " );
			for blob in row.iter() {
				if blob.0 == 0 {
					print!("   ");
				} else {
					print!( "{:x}{:x} ", blob.0 & 0xf, blob.1 & 0xf );
				}
			}
			println!( "]" );
        }
	}

	fn new_creature(&mut self) -> Blob {
		let mut rng = rand::thread_rng();
		loop {
			let x = rng.gen_range(0..DIM);
			let y = rng.gen_range(0..DIM); // DIM);
			if self.map[y][x] == (0,0) {
				// println!("Adding new creature");
				let name = self.next_name;
				self.next_name += 1;
				self.map[y][x] = (name, 0);
				break Blob{ 
					x, y, name,
					x0: x, y0: y,
					movx: 0.,
					movy: 0.,
					age: 0,
				}
			}
		}
	}
}

// We'll use the environment so we can account for collisions in the act. 
// (We'll also incidentally print from the environment.)
// (Probably should have just printed while cyclying through world.organisms.creature objects, but oh well. Just use a tuple for map and bob's your uncle.)
impl Environs for MyEnv {
	type Creature = Blob;
	fn new() -> Self { 
		Self{
			next_name: 1,
			map: [[(0,0); DIM]; DIM],
		} 
	}
}

struct Blob {
	age: usize,
	name: usize,
	movx: f32,
	movy: f32,
	x: usize,
	y: usize,
	x0: usize,
	y0: usize, 
}

impl Creature for Blob {
	type Env = MyEnv;
	type CCT = Self;
	
	fn new( env :&mut Self::Env, with :Vec<&Self::CCT> ) -> Self {
		let mut new = env.new_creature();
		if with.len() > 0 { // inherit name of first parent (the "mom" if you will, although siblings can spawn based on ALL parents locations, so "mom" doesn't necessarily make sense any more, unless we want to provide "nourishing" functions or something of that matter. The user can do that on their end. )
			new.name = with[0].name;
			env.map[new.y][new.x] = (new.name, 0); // Important! Change the name in the MAP (ugh)!
		} 
		new
	}

	fn die(&self, age :usize, _fitness :f32, env :&mut Self::Env) -> bool { 
		if age > Config::get().lifespan {
			env.map[ self.y ][ self.x ] = (0,0);
			true
		} else {
			false
		}
	}

	fn act( &mut self, env :&mut Self::Env ) -> f32 {
		let mut rng = rand::thread_rng();
		self.age += 1; // update local age

		// movx and movy are probability of moving in positive or negative direction
		let dx = if f32::abs(self.movx) > rng.gen_range(0.0..1.) {
			if self.movx > 0. { 1 } else { -1 }
		} else { 0 };
		let dy = if f32::abs(self.movy) > rng.gen_range(0.0..1.) {
			if self.movy > 0. { 1 } else { -1 }
		} else { 0 };

		// Do some bounds checking
		let newx = usize::min( isize::max(0, self.x as isize + dx) as usize, DIM-1 );
		let newy = usize::min( isize::max(0, self.y as isize + dy) as usize, DIM-1 );

 		// If cell is empty, let's move!
 		if env.map[newy][newx] == (0,0) { 
 			env.map[self.y][self.x] = (0,0);
 			self.x = newx;
 			self.y = newy;
 			env.map[self.y][self.x] = (self.name, self.age);
 		} else { // else just update the local age
			env.map[self.y][self.x].1 = self.age;
		}

		// Return Fitness: 

		// How far north have you traveled, vs how far you could have traveled? 
		let y_traveled:isize = self.y0 as isize - self.y as isize;
		let y_max = usize::min( self.y0, self.age );
		if y_max == 0 { 
			return 0. // haven't moved or born on the border, can't ascertain fitness
		} else {
			return y_traveled as f32 / y_max as f32
		}

	//	if self.y <= (DIM >> 2) { // upper quadrant
	//		(DIM - self.y) as f32 
	//	} else { 
	//		0.
	//	}
	}
	
	fn rx_input(&self, input :&str, _env :&Self::Env) -> f32 {
		match input {
			// "DT" => self.y as f32,
			"X" => self.x as f32 / (DIM-1) as f32,
			"Y" => self.y as f32 / (DIM-1) as f32,
			"DB" => (DIM - self.y) as f32 / DIM as f32,
			"DT" => self.y as f32 / (DIM-1) as f32,
			_ => { 
				println!("rx_input: no match found for: {}", input );
				return 0.
			},
		}
	}
	
	fn tx_output(&mut self, output :&str, value :f32, _env :&Self::Env) {
		match output { 
			"MOVX" => self.movx = math::tanh( value ), // -1 to 1 
			"MOVY" => self.movy = math::tanh( value ), // -1 to 1
			_ => println!("tx_output: no match found for: {}", output ),
		}
	}
}
