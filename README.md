# cm2rs
An advanced logic gate circuit builder and simulator, integrated with Circuit Maker 2's logic gate system.

> cm2rs is in early development - functional but buggy

## Features

- A Circuit schematic builder
- A built in pre-state solver
- Export/Import options for cm2 register strings
- A built in logic gate simulator

## Usage
Add to your `Cargo.toml`:
```toml
[dependencies]
cm2r = { git = "https://github.com/Hawkking-cloud/cm2rs" }
```

```rust
use cm2rs::*;

// Make a full adder in cm2rs
let mut cb = CircuitBuilder::new();

let a = cb.add_input_bit( (0,0,0), "a" );
let b = cb.add_input_bit( (0,0,1), "b" );
let cin = cb.add_input_bit(0,0,2);

let out = cb.add_output_bit( (3,0,1), "out" );
let cout = cb.add_output_bit( (3,0,3), "cout" );

let xor1 = cb.add_block(Block::XOR, (1,0,0) ); // a ^ b
let and1 = cb.add_block(Block::AND, (1,0,1) ); // a & b

cb.set_inputs( xor1, vec![ a, b ] );
cb.set_inputs( and1, vec![ a, b ] );

let xor2 = cb.add_block(Block::XOR, (2,0,0) ); // (a ^ b) ^ c
let and2 = cb.add_block(Block::AND, (2,0,1) ); // (a ^ b) & c
let or = cb.add_block(Block::OR, (1,0,2) ); // (a & b & c)

cb.set_inputs( xor2, vec![ xor1, cin ] );
cb.set_inputs( and2, vec![ xor1, cin ] );
cb.set_inputs( or, vec![ and1, and2 ] );

cb.set_input( out, xor2 );
cb.set_input( cout, or );

let cm2_string:String = cb.create_cm2();

println!("{}",cm2_string);
```



## Examples
> More examples to come
```bash
cargo run --example rca_adder
cargo run --example csa_adder
cargo run --example cla_adder
```

## Known issues
- Early problem with how simulating accepts active inputs between ticks


## Possible future commits
- Proper integration for every cm2 block lol
- Simulation compute upgrade (64b cpu simd operations)
- CM2 Building integration
- Block reformatting options
- Logic optimization options
- Size optimization options

## Planned examples
- every other known adder, maybe some of the lookup table ones
