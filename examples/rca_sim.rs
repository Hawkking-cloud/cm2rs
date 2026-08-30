#![allow(unused)]
use cm2rs::{
    circuit::{Block, BlockProxy, CircuitBuilder},
    sim::{BusKind, BusValue},
};

fn main() {
    let size = BusValue::U16(0).kind().width();

    let mut cb = CircuitBuilder::new();

    let input1 = cb.add_input_bus((0, 0, 0), (0, 0, 1), BusValue::U16(321), "input2");
    let input2 = cb.add_input_bus((1, 0, 0), (0, 0, 1), BusValue::U16(123), "input1");
    let input_cin = cb.add_input_bit((0,0,-2), "first_cin",BusValue::Bit(false));

    let output1 = cb.add_output_bus((10, 0, 0), (0, 0, 1), BusKind::U16, "output1");


    for i in 0..size {
        let y: f32 = i as f32;
        // full adder
        let xor1 = cb.add_block((2, 0, y), Block::XOR);
        cb.set_inputs(xor1, vec![input1[i], input2[i]]);
        let and1 = cb.add_block((3, 0, y), Block::AND);
        cb.set_inputs(and1, vec![input1[i], input2[i]]);
        let xor2 = cb.add_block((4, 0, y), Block::XOR);         
        let and2 = cb.add_block((5, 0, y), Block::AND);
        let cin = if i == 0 {
            input_cin
        } else {
            let ret = cb.position_hash((6, 0, y - 1.0)).unwrap();
            // dbg!(cb.blocks.get(ret.value()).unwrap().position);
            ret
        };
        cb.set_inputs(and2, vec![xor1, cin]);
        cb.set_inputs(xor2, vec![xor1, cin]);
        let or1 = cb.add_block((6, 0, y), Block::OR);
        cb.set_inputs(or1,vec![and1,and2]);
        cb.set_input(output1[i],xor2);
    }

    // cb.wire_parallel(input1,output1);

    // let mut sim = cb.create_sim();
    // for i in 0..127 {
    //     // if true {
    //     //     println!("{:?}", sim.get_output("output1"));
    //     // }
    //     sim.tick();
    // }
    println!("{:?}",cb.create_cm2());
    // still fluctuating

    // why is the output fluctating, need to prove less complicated circuits

    // println!("{:?}", sim.get_output("output1"));

    // 2 u8 input AND bus operator with output
    // let mut cb = CircuitBuilder::new();
    //
    // let input1 = cb.add_input_bus((0, 0, 0), (1, 0, 0), BusValue::U8(0b01100110), "input1");
    // let input2 = cb.add_input_bus((9, 0, 0), (1, 0, 0), BusValue::U8(0b11001100), "input2");
    //
    // let output1 = cb.add_output_bus((0, 4, 0), (1, 0, 0), BusKind::U8, "output1");
    //
    // for i in 0..8 {
    //     let and = cb.add_block((i, 2, 0), Block::XOR);
    //     cb.set_inputs(and, vec![input1[i], input2[i]]);
    //     cb.set_output(and, output1[i]);
    // }
    //
    // let mut sim = cb.create_sim();
    // for i in 0..(255 / 8) {
    //     let in1 = i * 8;
    //     let in2 = i * 8 / 3;
    //     // inputs arent updated in one tick
    //     sim.set_input("input1", BusValue::U8(in1));
    //     sim.set_input("input2", BusValue::U8(in2));
    //     sim.tick();
    //     sim.tick(); // 2 ticks to update the input
    //     println!(
    //         "input1: {} input2: {} output1: {:?}",
    //         in1,
    //         in2,
    //         sim.get_output("output1")
    //     );
    // }
    // dbg!(sim.get_output("output1"));
}
