#![allow(unused)]
use cm2rs::{
    circuit::{Block, BlockProxy, CircuitBuilder},
    sim::{BusKind, BusValue},
};

fn main() {
    let size: usize = 512;

    let mut cb = CircuitBuilder::new();

    let input1 = cb.add_input_bus((0, 0, 0), (0, 0, 1), BusValue::from_size(size), "in1");
    let input2 = cb.add_input_bus((1, 0, 0), (0, 0, 1), BusValue::from_size(size), "in2");
    let input_cin = cb.add_input_bit((0, 0, -2), "first_cin", BusValue::Bit(false));

    let output = cb.add_output_bus((10, 0, 0), (0, 0, 1), BusKind::Bus(size), "out");

    for i in 0..size {
        let y: f32 = i as f32;

        let xor1 = cb.add_block((2, 0, y), Block::XOR);
        cb.set_inputs(xor1, vec![input1[i], input2[i]]);

        let and1 = cb.add_block((3, 0, y), Block::AND);
        cb.set_inputs(and1, vec![input1[i], input2[i]]);

        let xor2 = cb.add_block((4, 0, y), Block::XOR);
        cb.set_output(xor2, output[i]);

        let and2 = cb.add_block((5, 0, y), Block::AND);

        let cin = if i == 0 {
            input_cin
        } else {
            let ret = cb.position_hash((6, 0, y - 1.0)).unwrap();
            ret
        };

        cb.set_inputs(and2, vec![xor1, cin]);
        cb.set_inputs(xor2, vec![xor1, cin]);

        let or1 = cb.add_block((6, 0, y), Block::OR);
        cb.set_inputs(or1, vec![and1, and2]);

    }

    let mut sim = cb.create_sim();


    sim.set_input("in1", BusValue::from_uint(size,101u64));
    sim.set_input("in2", BusValue::from_uint(size,27u64));

    sim.tick_until_stable("out", 1000,4);

    sim.print_output_uint("out");
    sim.print_output("out");
}
