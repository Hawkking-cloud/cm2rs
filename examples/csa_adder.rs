// csa_adder.rs   - Carry Select
// 2 ripple carry adders glued together to make carry-in switching 1 tick

use cm2rs::{
    circuit::{Block, CircuitBuilder,Cm2SaveOptions},
    sim::{BusKind, BusValue},
};


fn main() {
    let size: usize = 16;

    let mut cb = CircuitBuilder::new();

    let start_in1: u16 = 50;
    let start_in2: u16 = 25;

    let input1 = cb.add_input_bus(
        (-2, 1, 0),
        (0, 0, 1),
        BusValue::from_uint(size, start_in1),
        "in1",
    );
    let input2 = cb.add_input_bus(
        (-2, 0, 0),
        (0, 0, 1),
        BusValue::from_uint(size, start_in2),
        "in2",
    );
    let cin = cb.add_input_bit((-2, 0, -1), BusValue::Bit(false), "cin");

    let out = cb.add_output_bus((6 , 0, 0), (0, 0, 1), BusKind::Bus(size), "out");

    let cout = cb.add_output_bit((6, 0, size + 1), "cout");

    let static_false = cb.add_block((0, 0, size), Block::NOR);
    let static_true = cb.add_block((0, 1, size), Block::NOR);

    cb.set_input(static_false,static_true);

    let cin_nor = cb.add_block((-2, 1, -1), Block::NOR);
    cb.set_input(cin_nor, cin);

    // 2 full adders per bit
    // 1 2:1 mux per 2 full adder
    // 1 2:1 mux cout

    for bit in 0..size {
        // first full adder
        let fa1_xor1 = cb.add_block((0, 0, bit), Block::XOR);
        let fa1_and1 = cb.add_block((1, 0, bit), Block::AND);
        let fa1_xor2 = cb.add_block((0, 1, bit), Block::XOR);
        let fa1_and2 = cb.add_block((1, 1, bit), Block::AND);
        let fa1_or = cb.add_block((1, 2, bit), Block::OR);

        let fa1_carry = match bit {
            0 => static_false,
            _ => cb.position_hash((1, 2, bit - 1)).unwrap(), // last fa1_or
        };

        cb.set_inputs(fa1_xor1, vec![input1[bit], input2[bit]]);
        cb.set_inputs(fa1_and1, vec![input1[bit], input2[bit]]);
        cb.set_inputs(fa1_xor2, vec![fa1_xor1, fa1_carry]);
        cb.set_inputs(fa1_and2, vec![fa1_xor1, fa1_carry]);
        cb.set_inputs(fa1_or, vec![fa1_and1, fa1_and2]);

        // second full adder
        let fa2_xor1 = cb.add_block((2, 0, bit), Block::XOR);
        let fa2_and1 = cb.add_block((3, 0, bit), Block::AND);
        let fa2_xor2 = cb.add_block((2, 1, bit), Block::XOR);
        let fa2_and2 = cb.add_block((3, 1, bit), Block::AND);
        let fa2_or = cb.add_block((3, 2, bit), Block::OR);

        let fa2_carry = match bit {
            0 => static_true,
            _ => cb.position_hash((3, 2, bit - 1)).unwrap(), // last fa2_or
        };

        cb.set_inputs(fa2_xor1, vec![input1[bit], input2[bit]]);
        cb.set_inputs(fa2_and1, vec![input1[bit], input2[bit]]);
        cb.set_inputs(fa2_xor2, vec![fa2_xor1, fa2_carry]);
        cb.set_inputs(fa2_and2, vec![fa2_xor1, fa2_carry]);
        cb.set_inputs(fa2_or, vec![fa2_and1, fa2_and2]);

        // carry switch mux
        let mux_true = cb.add_block((4, 0, bit), Block::AND);
        let mux_false = cb.add_block((4, 1, bit), Block::AND);
        let mux_or = cb.add_block((4, 2, bit), Block::OR);

        cb.set_inputs(mux_true, vec![cin_nor, fa1_xor2]);
        cb.set_inputs(mux_false, vec![cin, fa2_xor2]);

        cb.set_inputs(mux_or, vec![mux_true, mux_false]);

        cb.add_output(mux_or, out[bit]);
    }

    // cout mux

    let cout_mux_true = cb.add_block((1, 0, size), Block::AND);
    let cout_mux_false = cb.add_block((2, 0, size), Block::AND);
    let cout_mux_or = cb.add_block((3, 0, size), Block::OR);

    cb.set_inputs(
        cout_mux_false,
        vec![cin_nor, cb.position_hash((1, 2, size - 1)).unwrap()],
    ); // last fa1_or
    cb.set_inputs(
        cout_mux_true,
        vec![cin, cb.position_hash((3, 2, size - 1)).unwrap()],
    ); // last fa2_or
    cb.set_inputs(cout_mux_or, vec![cout_mux_true, cout_mux_false]);
    cb.set_output(cout_mux_or, cout);

    // let mut sim = cb.create_sim();
    //
    // sim.tick_until_stable("out", 50, 5);
    //
    // sim.print_output_uint("out");

    let cm2_string = cb.create_cm2(Cm2SaveOptions{
        convert_ors: true,
        ..Default::default()
    });

    println!("{}",cm2_string);
}
