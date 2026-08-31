#![allow(non_snake_case)]
use cm2rs::{
    circuit::{Block, CircuitBuilder},
    sim::{BusKind, BusValue},
};

fn main() {
    let size: usize = 16; // bus size of adder

    let A_input:u8 = 41;
    let B_input:u8 = 67;

    let mut cb = CircuitBuilder::new();

    //     starting position  v v v    v v v  bus direction            bus label  v
    let A = cb.add_input_bus((0,0,0), (0,0,1), BusValue::from_uint(size,A_input),"a");

    let B = cb.add_input_bus((1,0,0), (0,0,1), BusValue::from_uint(size,B_input),"b");
    
    let Cin = cb.add_input_bit((1,0,-1), BusValue::Bit(false), "cin");
    //                             BusValue ^^^

    //                                   BusKind  vvv
    let S =  cb.add_output_bus((10,0,0),(0,0,1),BusKind::Bus(size),"out");

    let Cout = cb.add_output_bit((10,0,size+1),"cout");
    
    for z in 0..size {

        // A ⊕ B
        let input_xor = cb.add_block((2,0,z), Block::XOR);
        cb.set_inputs(input_xor,vec![A[z],B[z]]);
        // "set the inputs of block A to this vector of blocks, B"
        
        // A ∧ B
        let input_and = cb.add_block((3,0,z), Block::AND);
        cb.set_inputs(input_and,vec![A[z],B[z]]);
        
        // Cn 
        let carry_and = cb.add_block((4,0,z),Block::AND);

        let carry_xor = cb.add_block((5,0,z),Block::XOR);

        let full_adder_or = cb.add_block((6,0,z),Block::OR);
        
        cb.set_inputs(full_adder_or, vec![input_and, carry_and]);


        let carry = if z == 0 {
            Cin 
        } else {
            // position of the last full_adder_or
            cb.position_hash((6,0,z-1)).unwrap()
        };

        cb.set_inputs(carry_and,vec![input_xor,carry]);
        cb.set_inputs(carry_xor, vec![input_xor, carry]);
        
        // wiring to set output
        cb.set_output(carry_xor,S[z]);

        // wire last full_adder_or to cout 
        if z == size-1 {
            cb.set_output(full_adder_or, Cout);
        }
    }

    
    /*
    // active input doesnt function properly 

    let mut sim = cb.create_sim();

    sim.set_input("in1", BusValue::from_uint(size,101u64));
    sim.set_input("in2", BusValue::from_uint(size,27u64));

    sim.tick_until_stable("out", 1000,5);

    sim.print_output_uint("out");
    */

    let cm2_string = cb.create_cm2();

    println!("{}",cm2_string);

}
