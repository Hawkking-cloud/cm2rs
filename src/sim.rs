// sim.rs

use std::collections::HashMap;

use crate::circuit::BlockProxy;

#[repr(u8)]
pub enum Op {
    Nil = 0,
    Input = 1,
    Output = 2,
    NOR = 3,
    AND = 4,
    OR = 5,
    XOR = 6,
    XNOR = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusKind {
    Bit,
    U8,
    U16,
}

impl BusKind {
    pub fn width(&self) -> usize {
        match self {
            BusKind::Bit => 1,
            BusKind::U8 => 8,
            BusKind::U16 => 16,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BusValue {
    Bit(bool),
    U8(u8),
    U16(u16),
}

impl BusValue {
    pub fn kind(&self) -> BusKind {
        match self {
            BusValue::Bit(_) => BusKind::Bit,
            BusValue::U8(_) => BusKind::U8,
            BusValue::U16(_) => BusKind::U16,
        }
    }
}

pub type Label<'a> = &'a str;

pub struct Simulation<'a> {
    pub size: usize,
    pub state: Box<[u8]>,
    pub operations: Box<[u8]>,
    // pub input_map: Vec<(Box<[BlockProxy]>, Label<'a>, BusValue)>,
    pub input_table: Box<[Box<[usize]>]>,
    pub input_label_hash: HashMap<BlockProxy, Label<'a>>,
    pub input_hash: HashMap<Label<'a>, (Box<[BlockProxy]>, BusValue)>,
    pub output_hash: HashMap<Label<'a>, (Box<[BlockProxy]>, BusValue)>,
    pub outputs: HashMap<Label<'a>, BusValue>,
    pub output_label_hash: HashMap<BlockProxy, Label<'a>>,
}
pub struct SimulationDescriptor<'a> {
    pub size: usize,
    pub starting_state: Vec<u8>,
    pub starting_operations: Vec<u8>,
    pub starting_input_table: Vec<Vec<usize>>,
    // pub input_map: Vec<(Box<[BlockProxy]>, Label<'a>, BusValue)>,
    pub input_hash: HashMap<Label<'a>, (Box<[BlockProxy]>, BusValue)>,
    pub input_label_hash: HashMap<BlockProxy, Label<'a>>,
    pub output_hash: HashMap<Label<'a>, (Box<[BlockProxy]>, BusValue)>,
    pub output_label_hash: HashMap<BlockProxy, Label<'a>>,
}
impl<'a> Simulation<'a> {
    pub fn new(descriptor: &SimulationDescriptor<'a>) -> Self {
        assert_eq!(descriptor.size % 8, 0);
        let input_table: Box<[Box<[usize]>]> = descriptor
            .starting_input_table
            .iter()
            .map(|inner_slice| inner_slice.to_vec().into_boxed_slice())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        // let input_table: Box<[Box<[usize]>]> = descriptor
        //     .starting_input_table
        //     .clone()
        //     .iter_mut()
        //     .map(|vec| *vec.into_boxed_slice())
        //     .collect()
        //     .into_boxed_slice();

        // let input_table: Box<[Box<[usize]>]> = descriptor.starting_input_table.iter().map(|vec|vec.into_boxed_slice()).collect();
        Self {
            size: descriptor.size,
            state: descriptor.starting_state.clone().into_boxed_slice(),
            operations: descriptor.starting_operations.clone().into_boxed_slice(),
            input_hash: descriptor.input_hash.clone(),
            input_table: input_table,
            input_label_hash: descriptor.input_label_hash.clone(),
            outputs: HashMap::new(),
            output_hash: descriptor.output_hash.clone(),
            output_label_hash: descriptor.output_label_hash.clone(),
            // TODO: maybe look into consuming these values
        }
    }
    pub fn set_input(&mut self, label: Label<'a>, data: BusValue) {
        if let Some((block_map, value)) = self.input_hash.get_mut(label) {
            *value = data;
            block_map.iter().enumerate().for_each(|(bit_index, block)| {
                let i = block.value();
                let bit = match data {
                    BusValue::Bit(b) => b,
                    BusValue::U8(v) => ((v >> bit_index) & 1) != 0,
                    BusValue::U16(v) => ((v >> bit_index) & 1) != 0,
                };
                if bit {
                    self.state[i / 8] |= 1 << (i % 8);
                } else {
                    self.state[i / 8] &= !(1 << (i % 8));
                }
            });
        }
    }
    pub fn get_output(&self, label: Label<'a>) -> Option<&BusValue> {
        Some(&self.output_hash.get(label).unwrap().1)
    }

    // input_table type = 0: #0's, 1: #1's
    pub fn tick(&mut self) {
        self.outputs.clear();
        //TODO: init every other hashmaps

        // rewritten
        let input_table: Vec<(usize, usize)> = self
            .input_table
            .iter()
            .map(|inputs| {
                inputs.iter().fold((0usize, 0usize), |(inact, act), &b| {
                    let state = ((self.state[b / 8] >> (b % 8)) & 1) == 1;
                    if state {
                        (inact, act + 1)
                    } else {
                        (inact + 1, act)
                    }
                })
            })
            .collect();
        // let input_table: Vec<(usize, usize)> = self
        //     .input_table
        //     .iter()
        //     .map(|i| {
        //         i.iter().fold((0usize, 0usize), |(acc_zero, acc_one), &p| {
        //             let block_state = (self.state[p / 8] >> ((p % 8) as u32)) & 1 == 1;
        //             (
        //                 acc_zero + (!block_state as usize),
        //                 acc_one + (block_state as usize),
        //             )
        //         })
        //     })
        //     .collect();
        //NOR OP
        let mut nor_mask: Box<[u8]> = vec![0u8; self.size / 8].into_boxed_slice();
        let mut and_mask: Box<[u8]> = vec![0u8; self.size / 8].into_boxed_slice();
        let mut or_mask: Box<[u8]> = vec![0u8; self.size / 8].into_boxed_slice();
        let mut xor_mask: Box<[u8]> = vec![0u8; self.size / 8].into_boxed_slice();
        let mut xnor_mask: Box<[u8]> = vec![0u8; self.size / 8].into_boxed_slice();

        for (i, op) in self.operations.iter().enumerate() {
            if *op == Op::NOR as u8 && input_table[i].1 == 0 {
                nor_mask[i / 8] |= 1 << ((i % 8) as u32);
            }
            if *op == Op::AND as u8 && input_table[i].0 == 0 && input_table[i].1 != 0 {
                and_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::OR as u8 && input_table[i].1 != 0 {
                or_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::XOR as u8 && input_table[i].1 % 2 != 0 {
                xor_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::XNOR as u8 && input_table[i].1 % 2 == 0 {
                xnor_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::Output as u8 {
                // figure out if this is recieving any true input bits
                // dbg!(&input_table[i]);
                // its a circuit wiring issue
                //TODO: OPTIMIZE THIS

                // treat as or gate
                // accumulate output between output blocks (outputs wiped in beginning)
                // need to know which bit to manipulate
                // 1. find accum
                // 2. make bit mask
                // 3. accumulate bit mask

                // if is connected to anything active
                let bitdata = input_table[i].1 != 0; // OR
                // println!("{:?} 0's | {:?} 1's",input_table[i].0,input_table[i].1);
                // this isnt getting input from Op::Input
                if let Some(label) = self.output_label_hash.get(&BlockProxy::new(i)) {
                    if let Some((block_bus, bus_value)) = self.output_hash.get_mut(label) {
                        // dbg!();
                        //left off here
                        //why are output bus's not recieving correct input data from input busses
                        //input:op in sim.rs confirmed working
                        //guessing the bug is here
                        // this is directly true because of the starting_state calculation
                        // correct effect, wrong data
                        // dbg!(&input_table[i]); // input_table is never (0,1) for some reason
                        //   rewrite input_table logic
                        //   rewrite operation input logic

                        //if bool
                        //  set value to bitdata
                        //if u8
                        //  set bool at bit_index to bitdata

                        // let current_output:Option<&BusValue> = self.outputs.get(label);
                        let index = block_bus
                            .iter()
                            .position(|b| b == &BlockProxy::new(i))
                            .expect("uhoh");
                        match bus_value {
                            BusValue::Bit(b) => {
                                *b = bitdata;
                            }
                            BusValue::U8(v) => {
                                if bitdata {
                                    *v |= 1 << index
                                } else {
                                    *v &= !(1 << index)
                                }
                            }
                            BusValue::U16(v) => {
                                if bitdata {
                                    *v |= 1 << index
                                } else {
                                    *v &= !(1 << index)
                                }
                            }
                        }
                        // let new_val:BusValue<> = match bus_kind {
                        //     BusValue::Bool(b) => bitdata||(*b as bool),
                        //     BusValue::U8(v) => v,
                        // };
                        // let new_val = match bus_kind {
                        //     BusKind::Bool => BusValue::Bool(bitdata),
                        //     BusKind::U8 => {
                        //         // actively fetch and compose current output
                        //         // output accumulators?
                        //     }
                        // };
                        // dbg!(new_val);
                        // self.outputs.insert(label, new_val);
                    }
                }

                if bitdata {
                    or_mask[i / 8] |= 1 << (i % 8);
                }
            }
            if *op == Op::Input as u8 {
                // dbg!(&input_table[i]);
                let bitdata: bool = (input_table[i].1 != 0)
                    || self
                        .input_label_hash
                        .get(&BlockProxy::new(i))
                        .and_then(|label| {
                            // println!("{:?}",label);
                            let (block_bus, value) = self.input_hash.get(label).unwrap();
                            let bit_index =
                                block_bus.iter().position(|&x| x == BlockProxy::new(i))?;
                            Some(match value {
                                BusValue::Bit(b) => *b,
                                BusValue::U8(byte) => {
                                    // println!(
                                    //     "{:?} {:?} {:?}",
                                    //     bit_index,
                                    //     value,
                                    //     (byte >> bit_index) & 1 != 0
                                    // );
                                    (byte >> bit_index) & 1 != 0
                                },
                                BusValue::U16(byte) => {
                                    (byte >> bit_index) & 1 != 0
                                }
                            })
                        })
                        .unwrap_or(false);
                // correctly pushing the data to state
                if bitdata {
                    or_mask[i / 8] |= 1 << (i % 8);
                }
            }
        }

        self.state.iter_mut().enumerate().for_each(|(i, b)| {
            *b = (*b ^ nor_mask[i] ^ and_mask[i] ^ xor_mask[i] ^ xnor_mask[i]) | or_mask[i]
        });
        // for (i, op) in self.operations.iter().enumerate() {
        //     if *op == Op::Debug as u8 {
        //         self.debug_states
        //             .push(self.state[i / 8] | 1 << (i % 8) & 1 == 1);
        //
        //     }
        // }
    }
}
