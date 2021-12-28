# gworld
Rust library for genetic algorithms

## Usage tips
- [world.rs](./src/world.rs) defines the primary objects/traits. 

- World contains 1 environment and many creatures, which you will define via the Environs and Creature traits. 

- When setting the config, a "functional" chromosome is defined as a collection of genes that describe a full path from input to output. Ultimately, the idea is that breeding algorithms may be enhanced by mating organisms with similar chromosomal makeup. 

## Example

It doesn't do much, but it evolves (and compiles).

```rust:examples/blob.rs```
