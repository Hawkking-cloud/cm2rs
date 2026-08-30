use cm2rs::{
    circuit::{Block, CircuitBuilder},
    sim::{BusKind, BusValue},
};

fn main() {
    let size: usize = 8;

    let mut cb = CircuitBuilder::new();

    //OR OP
    let or_in = cb.add_input_bus((2,0,0),(0,0,1),BusValue::new_bus(size),"or");
    let or_op = cb.add_operator_bus((1,0,1),(0,0,1),BusKind::Bus(size),Block::OR);
    let or_out = cb.add_output_bus((0,0,0), (0,0,1), BusKind::Bus(size), "or");
    cb.wire_parallel(or_in, &or_op);
    cb.wire_parallel(or_op, &or_out);

    let mut sim = cb.create_sim();
    sim.tick();
    sim.print_output("or");
}
