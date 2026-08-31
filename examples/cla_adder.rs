use cm2rs::{
    circuit::{Block, CircuitBuilder},
    sim::{BusKind, BusValue},
};


//TODO: rename these to the actual proper logic gate names
fn main() {
    let size: usize = 16;

    let mut cb = CircuitBuilder::new();

    // let start_in1: u64 = 12345678;
    // let start_in2: u64 = 87654321;

    let input1 = cb.add_input_bus((-1, 1, 0), (0, 0, 1), BusValue::from_size(size), "in1");
    let input2 = cb.add_input_bus((-1, 0, 0), (0, 0, 1), BusValue::from_size(size), "in2");
    let cin = cb.add_input_bit((-1, 0, -1), BusValue::Bit(false), "cin");

    let output1 = cb.add_output_bus((3 + size + 3, 0, 0), (0, 0, 1), BusKind::Bus(size), "out");

    let cout = cb.add_output_bit((3 + size + 3, 0, size + 1), "cout");

    // let cout = cb.add_output_bit((0,0,-2),"cout");

    let cin_nor = cb.add_block((1, 0, -1), Block::NOR);
    cb.set_input(cin_nor, cin);

    for i in 0..size {
        let inp_nor = cb.add_block((1, 0, i), Block::NOR);
        cb.set_inputs(inp_nor, vec![input1[i], input2[i]]);

        let inp_nand = cb.add_block((2, 0, i), Block::NAND);
        cb.set_inputs(inp_nand, vec![input1[i], input2[i]]);

        //

        let prop_xor = cb.add_block((1 + size + 2, 0, i), Block::XOR);
        cb.set_inputs(prop_xor, vec![input1[i], input2[i]]);
        let out_xor = cb.add_block((1 + size + 3, 0, i), Block::XOR);
        cb.set_input(out_xor, prop_xor);
        cb.set_output(out_xor, output1[i]);
        if i > 0 {
            let idx_nor = cb.add_block((1 + size + 1, 0, i), Block::NOR);

            let last_input_nor = cb.position_hash((1, 0, i - 1)).unwrap();

            cb.add_input(idx_nor, last_input_nor);
            cb.add_output(idx_nor, out_xor);

            for j in 0..i {
                let jdx_and = cb.add_block((3 + j, 0, i), Block::AND);
                let jdx_in_nor = cb
                    .position_hash((1, 0, (i as i32) - (j as i32) - 2))
                    .unwrap();
                cb.add_input(jdx_and, jdx_in_nor);
                for l in 0..=j {
                    let ldx_inp = cb.position_hash((2, 0, i - l - 1)).unwrap();
                    cb.add_input(jdx_and, ldx_inp);
                }
                cb.add_output(jdx_and, idx_nor);
            }
        } else {
            cb.add_input(out_xor, cin);
        }
    }
    let cout_nor = cb.add_block((3 + size + 1, 0, size), Block::NOR);

    let final_input_nor = cb.position_hash((1, 0, size - 1)).unwrap();

    cb.add_input(cout_nor, final_input_nor);
    cb.add_output(cout_nor, cout);

    // second pass for cout 

    for i in 0..size {
        let jdx_and = cb.add_block((3 + i, 0, size), Block::AND);
        let jdx_in_xor = match size - i {
            1 => cin_nor,
            _ => final_input_nor,
        };
        cb.add_input(jdx_and, jdx_in_xor);
        for j in 0..i {
            let ldx_inp_nand = cb.position_hash((2, 0, size - j - 1)).unwrap();
            cb.add_input(jdx_and, ldx_inp_nand);
        }
    }

    println!("{}", cb.create_cm2());


    /*
    let schematic = cb.make_sim_schematic();

    let mut sim = cb.create_sim();

    sim.tick_until_stable("out", 500, 5);

    println!("{}",sim.snapshot_cm2(schematic));
    */

}

