use cm2rs::{
    circuit::{Block, CircuitBuilder},
    sim::{BusKind, BusValue},
};

fn main() {
    let size: usize = 8;

    let mut cb = CircuitBuilder::new();
    
    let in1:BusValue = BusValue::Bus((size,vec![0b00001100].into_boxed_slice()));
    let in2:BusValue = BusValue::Bus((size,vec![0b00001111].into_boxed_slice()));

    //OR OP
    let or_in1 = cb.add_input_bus((2,0,0),(0,0,1),in1.clone(),"or1");
    let or_in2 = cb.add_input_bus((3,0,0),(0,0,1),in2.clone(),"or2");
    let or_op = cb.add_operator_bus((1,0,1),(0,0,1),BusKind::Bus(size),Block::AND);
    let or_out = cb.add_output_bus((0,0,0), (0,0,1), BusKind::Bus(size), "or");
    cb.wire_parallel(or_in1, &or_op);
    cb.wire_parallel(or_in2, &or_op);
    cb.wire_parallel(or_op, &or_out);

    let mut sim = cb.create_sim();
    let steps= 1;
    // issue with simulating NOR
    // starting state is correct
    for _ in 0..steps {
        sim.tick();
    }
    
    // sim.print_output("or");
    println!("{:?}",sim.state);
}
