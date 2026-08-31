use cm2rs::{
    circuit::{CircuitBuilder,Cm2SaveOptions},
};

fn main() {
    let cm2_string = std::fs::read_to_string("assets/test.cm2").unwrap();
    let cb = CircuitBuilder::from_cm2(cm2_string);

    let ors_to_nodes = cb.create_cm2(Cm2SaveOptions {
        optimize_size: false,
        round_position_floats: false,
        grid_scale: 2,
        convert_ors: true,
    });

    println!("{}",ors_to_nodes);
}
