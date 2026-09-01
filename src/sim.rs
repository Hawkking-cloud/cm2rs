// sim.rs
#![allow(clippy::manual_is_multiple_of)] // shutup lsp

use std::collections::HashMap;

use crate::circuit::BlockProxy;

#[repr(u8)]
pub enum Op {
    Nil = 0,
    Input = 1,
    Output = 2,
    NOR = 3,
    AND = 4,
    NAND = 5,
    OR = 6,
    XOR = 7,
    XNOR = 8,
    CM2 = 9,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusKind {
    Bit,
    U4,
    U8,
    U16,
    U32,
    U64,
    Bus(usize),
}

impl BusKind {
    pub fn width(&self) -> usize {
        match self {
            BusKind::Bit => 1,
            BusKind::U4 => 8,
            BusKind::U8 => 8,
            BusKind::U16 => 16,
            BusKind::U32 => 32,
            BusKind::U64 => 64,
            BusKind::Bus(size) => *size,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BusValue {
    Bit(bool),
    U4(u8), //TODO: maybe do something about this
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Bus((usize, Box<[u8]>)),
}

pub trait ToBusBytes {
    fn to_ne_bytes_vec(self) -> Vec<u8>;
}

macro_rules! impl_to_bus_bytes {
    ($($t:ty),*) => {
        $(
            impl ToBusBytes for $t {
                fn to_ne_bytes_vec(self) -> Vec<u8> {
                    self.to_ne_bytes().to_vec()
                }
            }
        )*
    };
}

impl_to_bus_bytes!(u8, u16, u32, u64, u128, usize);

impl BusValue {
    pub fn new_bus(size: usize, mut start_data: Vec<u8>) -> Self {
        start_data.resize(size.div_ceil(8), 0u8);
        BusValue::Bus((size, start_data.into_boxed_slice()))
    }

    pub fn from_size(size: usize) -> Self {
        BusValue::Bus((size, vec![0u8; size.div_ceil(8)].into_boxed_slice()))
    }

    pub fn from_uint<T: ToBusBytes>(size: usize, val: T) -> Self {
        let bytes = val.to_ne_bytes_vec();
        BusValue::new_bus(size, bytes)
    }

    pub fn kind(&self) -> BusKind {
        match self {
            BusValue::Bit(_) => BusKind::Bit,
            BusValue::U4(_) => BusKind::U4,
            BusValue::U8(_) => BusKind::U8,
            BusValue::U16(_) => BusKind::U16,
            BusValue::U32(_) => BusKind::U32,
            BusValue::U64(_) => BusKind::U64,
            BusValue::Bus((size, _)) => BusKind::Bus(*size),
        }
    }
}

pub struct SnapshotInfo<'a> {
    pub location_hash:HashMap<usize,(f32,f32,f32)>,
    pub input_map:HashMap<usize,(Box<[usize]>,Label<'a>)>,
    pub output_map:HashMap<usize,(Box<[usize]>,Label<'a>)>,
    // pub cm2_map:HashMap<usize,u8>,
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

macro_rules! set_bit {
    ($v:expr, $index:expr, $bitdata:expr) => {
        if $bitdata {
            *$v |= 1 << $index
        } else {
            *$v &= !(1 << $index)
        }
    };
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
            input_table,
            input_hash: descriptor.input_hash.clone(),
            input_label_hash: descriptor.input_label_hash.clone(),
            outputs: HashMap::new(),
            output_hash: descriptor.output_hash.clone(),
            output_label_hash: descriptor.output_label_hash.clone(),
            // TODO: maybe look into consuming these values
        }
    }
    pub fn set_input(&mut self, label: Label<'a>, data: BusValue) {
        if let Some((block_map, value)) = self.input_hash.get_mut(label) {
            block_map.iter().enumerate().for_each(|(bit_index, block)| {
                let i = block.value();
                let bit = match &data {
                    BusValue::Bit(b) => *b,
                    BusValue::U4(v) => ((v >> bit_index) & 1) != 0,
                    BusValue::U8(v) => ((v >> bit_index) & 1) != 0,
                    BusValue::U16(v) => ((v >> bit_index) & 1) != 0,
                    BusValue::U32(v) => ((v >> bit_index) & 1) != 0,
                    BusValue::U64(v) => ((v >> bit_index) & 1) != 0,
                    BusValue::Bus((_, bus_data)) => {
                        let byte_index = bit_index / 8;
                        let bit_in_byte = bit_index % 8;
                        ((bus_data[byte_index] >> bit_in_byte) & 1) != 0
                    }
                };
                if bit {
                    self.state[i / 8] |= 1 << (i % 8);
                } else {
                    self.state[i / 8] &= !(1 << (i % 8));
                }
            });
            *value = data;
        }
    }
    pub fn verify_input(&self, label: Label<'a>, desired_value: BusValue) -> bool {
        desired_value
            == self
                .input_hash
                .get(label)
                .expect("invalid input label")
                .1
                .clone()
    }

    pub fn get_output(&self, label: Label<'a>) -> Option<&BusValue> {
        Some(&self.output_hash.get(label).unwrap().1)
    }
    pub fn print_output(&self, label: Label<'a>) {
        println!("{:?}", &self.output_hash.get(label).unwrap().1);
    }
    pub fn print_output_uint(&self, label: Label<'a>) {
        if let BusValue::Bus((_, data)) = self.get_output(label).unwrap() {
            let mut digits: Vec<u8> = vec![0];

            for &byte in data.iter().rev() {
                let mut carry = byte as u32;
                for d in digits.iter_mut() {
                    let val = *d as u32 * 256 + carry;
                    *d = (val % 10) as u8;
                    carry = val / 10;
                }
                while carry > 0 {
                    digits.push((carry % 10) as u8);
                    carry /= 10;
                }
            }

            while digits.len() > 1 && *digits.last().unwrap() == 0 {
                digits.pop();
            }

            println!(
                "{}",
                digits
                    .iter()
                    .rev()
                    .map(|d| (d + b'0') as char)
                    .collect::<String>()
            );
        }
    }
    
    pub fn snapshot_cm2(
        &mut self,
        info:SnapshotInfo,
    ) -> String {
        let mut buf = String::new();
        self.operations.iter().enumerate().for_each(|(i,op)|{
            let block_state = ((self.state[i/8] >> (i%8)) & 1) != 0;
            let (x,y,z)= info.location_hash.get(&i).expect("invalid snapshot info");

            buf.push_str(
                match op {
                    1 => "0,",
                    2 => "15,",
                    3 => "0,",
                    4 => "1,",
                    5 => "10,",
                    6 => "2,",
                    7 => "3,",
                    8 => "11,",
                    9 => {
                        // resolve saved cm2 block id, fallback to "15," (node)
                        "15,"
                    }
                    _ => "15,"
                }
            );
            buf.push_str(
                if block_state {
                    "0,"
                } else {
                    "1,"
                }
            );
            buf.push_str(&format!("{},{},{},;",x,y,z));
        });
        if !self.operations.is_empty() {
            buf.pop();
        }
        buf.push('?');
        self.input_table.iter().enumerate().for_each(|(i,from)|{
            buf.push_str(&format!("{:?},{:?};",from,i));
        });
        if !self.input_table.is_empty() {
            buf.pop();
        }
        buf.push_str("??");


        buf
    }

    pub fn tick_until_stable(
        &mut self,
        label: Label<'a>,
        max_iterations: usize,
        trend_count: usize,
    ) {
        let mut last_value: BusValue = self
            .get_output(label)
            .expect("invalid output label")
            .clone();
        let mut stable_trend: usize = 0;
        let mut num_iterations: usize = 0;
        for i in 0..max_iterations {
            self.tick();
            let current_value: &BusValue = self.get_output(label).expect("invalid output label");
            if last_value == *current_value {
                stable_trend += 1;
            } else {
                stable_trend = 0;
            }
            if stable_trend > trend_count {
                num_iterations = i + 1;
                break;
            }
            last_value = current_value.clone();
        }
        if stable_trend <= trend_count {
            println!("[{:?}] max_iterations hit: {}", label, max_iterations);
        } else {
            println!(
                "[ {:#?} ]: stable after {} iterations",
                label, num_iterations
            );
        }
    }

    // input_table type = 0: #0's, 1: #1's
    pub fn tick(&mut self) {
        self.outputs.clear();
        // rewritten
        // let input_table: Vec<(usize, usize)> = self
        //     .input_table
        //     .iter()
        //     .map(|inputs| {
        //         inputs.iter().fold((0usize, 0usize), |(inact, act), &b| {
        //             let state = ((self.state[b / 8] >> (b % 8)) & 1) == 1;
        //             if state {
        //                 (inact, act + 1)
        //             } else {
        //                 (inact + 1, act)
        //             }
        //         })
        //     })
        //     .collect();

        // rewrite to override input blocks with bus value bit index bool
        let input_table: Vec<(usize, usize)> = self
            .input_table
            .iter()
            .map(|inputs| {
                inputs.iter().fold((0usize, 0usize), |(inact, act), &b| {
                    let state = if self.operations[b] == Op::Input as u8 {
                        let input_label = self
                            .input_label_hash
                            .get(&BlockProxy::new(b))
                            .expect("invalid input");
                        let (block_map, bus_value) =
                            self.input_hash.get(input_label).expect("invalid input");
                        let bit_index = block_map
                            .iter()
                            .position(|&idx_block| idx_block.value() == b)
                            .unwrap();

                        if let BusValue::Bus((_, value)) = bus_value {
                            static mut I: u32 = 1;
                            unsafe {
                                I += 1;
                                if I % 8 == 0 {}
                            }

                            ((value[bit_index / 8] >> (bit_index % 8)) & 1) == 1
                        } else {
                            false
                        }
                    } else {
                        ((self.state[b / 8] >> (b % 8)) & 1) == 1
                    };
                    if state {
                        (inact, act + 1)
                    } else {
                        (inact + 1, act)
                    }
                })
            })
            .collect();

        let mask_size = self.size / 8;
        let mut nor_mask: Box<[u8]> = vec![0u8; mask_size].into_boxed_slice();
        let mut and_mask: Box<[u8]> = vec![0u8; mask_size].into_boxed_slice();
        let mut nand_mask: Box<[u8]> = vec![0u8; mask_size].into_boxed_slice();
        let mut or_mask: Box<[u8]> = vec![0u8; mask_size].into_boxed_slice();
        let mut xor_mask: Box<[u8]> = vec![0u8; mask_size].into_boxed_slice();
        let mut xnor_mask: Box<[u8]> = vec![0u8; mask_size].into_boxed_slice();

        for (i, op) in self.operations.iter().enumerate() {
            if *op == Op::OR as u8 && input_table[i].1 != 0 {
                or_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::NOR as u8 && input_table[i].1 == 0 {
                nor_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::AND as u8 && input_table[i].0 == 0 {
                and_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::NAND as u8 && input_table[i].0 != 0 {
                nand_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::XOR as u8 && input_table[i].1 % 2 != 0 {
                xor_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::XNOR as u8 && input_table[i].1 % 2 == 0 {
                xnor_mask[i / 8] |= 1 << (i % 8);
            }
            if *op == Op::Output as u8 {
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
                if let Some(label) = self.output_label_hash.get(&BlockProxy::new(i))
                    && let Some((block_bus, bus_value)) = self.output_hash.get_mut(label)
                {
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
                        BusValue::U4(v) => set_bit!(v, index, bitdata),
                        BusValue::U8(v) => set_bit!(v, index, bitdata),
                        BusValue::U16(v) => set_bit!(v, index, bitdata),
                        BusValue::U32(v) => set_bit!(v, index, bitdata),
                        BusValue::U64(v) => set_bit!(v, index, bitdata),
                        BusValue::Bus((_, data)) => {
                            let byte_index = index / 8;
                            let bit_in_byte = index % 8;
                            if bitdata {
                                data[byte_index] |= 1 << bit_in_byte;
                            } else {
                                data[byte_index] &= !(1 << bit_in_byte);
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

                if bitdata {
                    or_mask[i / 8] |= 1 << (i % 8);
                }
            }
            if *op == Op::Input as u8 {
                // dbg!(&input_table[i]);
                // input label hash is correct
                let bitdata: bool = self
                    .input_label_hash
                    .get(&BlockProxy::new(i))
                    .and_then(|label| {
                        // println!("{:?}",label);
                        let (block_bus, value) = self.input_hash.get(label).unwrap();
                        // value here is correct
                        let bit_index = block_bus.iter().position(|&x| x == BlockProxy::new(i))?;
                        Some(match value {
                            BusValue::Bit(bit) => *bit,
                            BusValue::U4(value) => (value >> bit_index) & 1 != 0,
                            BusValue::U8(value) => (value >> bit_index) & 1 != 0,
                            BusValue::U16(value) => (value >> bit_index) & 1 != 0,
                            BusValue::U32(value) => (value >> bit_index) & 1 != 0,
                            BusValue::U64(value) => (value >> bit_index) & 1 != 0,

                            // correct output bits
                            BusValue::Bus((_, bus_data)) => {
                                let byte_index = bit_index / 8;
                                let bit_in_byte = bit_index % 8;
                                ((bus_data[byte_index] >> bit_in_byte) & 1) != 0
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
            *b = (*b ^ xnor_mask[i])
                | or_mask[i]
                | nor_mask[i]
                | and_mask[i]
                | xor_mask[i]
                | nand_mask[i]
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
