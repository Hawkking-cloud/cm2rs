// circuit.rs

use std::collections::HashMap;

use crate::sim::{BusKind, BusValue, Label, Op, Simulation, SimulationDescriptor};

#[derive(PartialEq)]
pub enum Block {
    NOR,
    AND,
    OR,
    XOR,
    XNOR,
    Input,
    Output,
}

impl Block {
    pub fn to_op(&self) -> u8 {
        let op = match self {
            Block::NOR => Op::NOR,
            Block::AND => Op::AND,
            Block::OR => Op::OR,
            Block::XOR => Op::XOR,
            Block::XNOR => Op::XNOR,
            Block::Output => Op::Output,
            Block::Input => Op::Input,
        };
        op as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockProxy(usize);

impl BlockProxy {
    pub fn new(value: usize) -> Self {
        BlockProxy(value)
    }
    pub fn value(self) -> usize {
        self.0
    }
}

pub trait AsF32 {
    fn as_f32(self) -> f32;
}

macro_rules! impl_as_f32 {
    ($($t:ty),* $(,)?) => {
        $(
            impl AsF32 for $t {
                #[inline(always)]
                fn as_f32(self) -> f32 {
                    self as f32
                }
            }
        )*
    };
}

impl_as_f32!(f32, f64, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

pub type BlockPosition = (f32, f32, f32);
pub type BlockPositionKey = (i32, i32, i32);

pub trait IntoBlockPosition {
    fn into_pos(self) -> BlockPosition;
}

pub trait IntoBlockPositionKey {
    fn into_key(self) -> BlockPositionKey;
}
impl<A, B, C> IntoBlockPosition for (A, B, C)
where
    A: AsF32,
    B: AsF32,
    C: AsF32,
{
    #[inline(always)]
    fn into_pos(self) -> BlockPosition {
        (self.0.as_f32(), self.1.as_f32(), self.2.as_f32())
    }
}

impl<A, B, C> IntoBlockPositionKey for (A, B, C)
where
    A: AsF32,
    B: AsF32,
    C: AsF32,
{
    #[inline(always)]
    fn into_key(self) -> BlockPositionKey {
        let pos: BlockPosition = self.into_pos();
        (
            pos.0.floor() as i32,
            pos.1.floor() as i32,
            pos.2.floor() as i32,
        )
    }
}

pub struct BlockData {
    pub r#type: Block,
    pub position: BlockPosition,
    inputs: Vec<BlockProxy>, // indices
}
pub struct CircuitBuilder<'a> {
    pub blocks: Vec<BlockData>,
    block_registry: HashMap<BlockPositionKey, BlockProxy>,
    // input_map: Vec<(Box<[BlockProxy]>, Label<'a>, BusValue)>,
    input_hash: HashMap<Label<'a>, (Box<[BlockProxy]>, BusValue)>,
    input_label_hash: HashMap<BlockProxy, Label<'a>>,
    // output_map: Vec<(Box<[BlockProxy]>, Label<'a>, BusKind)>,
    output_hash: HashMap<Label<'a>, (Box<[BlockProxy]>, BusValue)>,
    output_label_hash: HashMap<BlockProxy, Label<'a>>,
}

impl<'a> CircuitBuilder<'a> {
    pub fn new() -> CircuitBuilder<'a> {
        Self {
            blocks: Vec::new(),
            block_registry: HashMap::new(),
            input_hash: HashMap::new(),
            input_label_hash: HashMap::new(),
            output_hash: HashMap::new(),
            output_label_hash: HashMap::new(),
        }
    }
    pub fn add_block<P>(&mut self, position: P, r#type: Block) -> BlockProxy
    where
        P: IntoBlockPosition,
    {
        let position: BlockPosition = position.into_pos();
        let index = self.blocks.len();
        self.blocks.push(BlockData {
            r#type: r#type,
            position: (-position.0,position.1,position.2),
            inputs: Vec::new(),
        });
        self.block_registry
            .insert(position.into_key(), BlockProxy::new(index));
        BlockProxy(index)
    }
    pub fn add_input_bit<P:IntoBlockPosition>(&mut self, position: P, label: Label<'a>, value:BusValue) -> BlockProxy {
        let position:BlockPosition = position.into_pos();
        let position:(f32,f32,f32) = (-position.0,position.1,position.2);
         let block = self.add_block(position,Block::Input);
        self.input_label_hash.insert(block,label);
        self.input_hash.insert(label, (Box::new([block.clone()]), value));
        block
    }
    pub fn add_output_bit<P>(&mut self, position: P, label: Label<'a>) -> BlockProxy
    where
        P: IntoBlockPosition + Copy,
    {
        let index = self.blocks.len();
        self.blocks.push(BlockData {
            r#type: Block::Output,
            position: position.into_pos(),
          inputs: Vec::new(),
        });
        self.output_hash.insert(
            label,
            (Box::new([BlockProxy::new(index)]), BusValue::Bit(false)),
        );
        self.block_registry
            .insert(position.into_pos().into_key(), BlockProxy::new(index));
        BlockProxy(index)
    }
    pub fn add_input_bus(
        &mut self,
        position: impl IntoBlockPosition,
        inc: (i32, i32, i32),
        bus_value: BusValue,
        label: Label<'a>,
    ) -> Box<[BlockProxy]> {
        let position: BlockPosition = position.into_pos();
        let bus_kind = bus_value.kind();
        let mut out_bus: Box<[BlockProxy]> =
            vec![BlockProxy::new(0); bus_kind.width()].into_boxed_slice();

        for i in 0..bus_kind.width() {
            let index_position = (
                position.0 + (inc.0 * i as i32) as f32,
                position.1 + (inc.1 * i as i32) as f32,
                position.2 + (inc.2 * i as i32) as f32,
            );
            out_bus[i] = self.add_block(index_position, Block::Input);
        }

        out_bus.iter().for_each(|&p| {
            self.input_label_hash.insert(p, label);
        });

        self.input_hash.insert(label, (out_bus.clone(), bus_value));
        out_bus
    }
    pub fn add_output_bus(
        &mut self,
        position: impl IntoBlockPosition,
        inc: (i32, i32, i32),
        bus_kind: BusKind,
        label: Label<'a>,
    ) -> Box<[BlockProxy]> {
        let position: BlockPosition = position.into_pos();
        let mut out_bus: Box<[BlockProxy]> =
            vec![BlockProxy(0usize); bus_kind.width()].into_boxed_slice();

        for i in 0..bus_kind.width() {
            let index_position = (
                position.0 + (inc.0 * i as i32) as f32,
                position.1 + (inc.1 * i as i32) as f32,
                position.2 + (inc.2 * i as i32) as f32,
            );
            out_bus[i] = self.add_block(index_position, Block::Output);
        }
        out_bus.iter().for_each(|&p| {
            self.output_label_hash.insert(p, label);
        });

        self.output_hash.insert(
            label,
            (
                out_bus.clone(),
                match bus_kind {
                    BusKind::Bit=> BusValue::Bit(false),
                    BusKind::U8 => BusValue::U8(0),
                    BusKind::U16 => BusValue::U16(0),
                },
            ),
        ); // TODO: change to bus_value

        out_bus
    }
    pub fn set_input(&mut self, block1: BlockProxy, block2: BlockProxy) {
        self.blocks
            .get_mut(block1.value())
            .expect("invalid block1 argument")
            .inputs = vec![block2];
    }
    pub fn set_inputs(&mut self, block1: BlockProxy, inputs: Vec<BlockProxy>) {
        self.blocks
            .get_mut(block1.value())
            .expect("invalid block1 argument")
            .inputs = inputs;
    }
    pub fn set_output(&mut self, block1: BlockProxy, block2: BlockProxy) {
        self.blocks
            .get_mut(block2.value())
            .expect("invalid second block argument")
            .inputs = vec![block1];
    }
    pub fn set_outputs(&mut self, block1: BlockProxy, outputs: Vec<BlockProxy>) {
        outputs.iter().for_each(|out| {
            self.blocks
                .get_mut(out.value())
                .expect("invalid block in outputs")
                .inputs = vec![block1];
        });
    }
    pub fn add_input(&mut self, block1: BlockProxy, block2: BlockProxy) {
        self.blocks
            .get_mut(block1.value())
            .expect("block1 invalid")
            .inputs
            .push(block2);
    }
    pub fn add_inputs(&mut self, block1: BlockProxy, mut inputs: Vec<BlockProxy>) {
        self.blocks
            .get_mut(block1.value())
            .expect("block1 invalid")
            .inputs
            .append(&mut inputs);
    }

    pub fn wire_parallel(&mut self, bus1: Box<[BlockProxy]>, bus2: Box<[BlockProxy]>) {
        bus2.iter().enumerate().for_each(|(i, block)| {
            self.blocks
                .get_mut(block.value())
                .expect("invalid bus2 block")
                .inputs
                .push(bus1[i])
        });
    }
    pub fn position_hash<P>(&self, position: P) -> Option<BlockProxy>
    where
        P: IntoBlockPositionKey,
    {
        // dbg!(&position.into_key());
        // None
        self.block_registry.get(&position.into_key()).copied()
    }
    fn fetch_via_proxy(&self,block_proxy: &BlockProxy) -> Option<&BlockData> {
        //TODO: replace all self.block.get() calls with this method
        self.blocks.get(block_proxy.value())
    }
    fn solve_starting_state(&self, path: &mut Vec<BlockProxy>, map: &mut HashMap<BlockProxy,bool>,block_proxy: &BlockProxy) -> bool {

        if let Some(&cached) = map.get(block_proxy) {
            return cached;
        }
        if path.contains(block_proxy) {
            return false;
        }

        let block = self.fetch_via_proxy(block_proxy).expect("invalid proxy");
        let block_type = &block.r#type;


        path.push(*block_proxy);

        // resolve predictable cases
        let result = *block_type == Block::NOR && block.inputs.len()==0 ||
        // resolve direct cases
        *block_type == Block::NOR && {
            !block.inputs.iter().any(|proxy_i|self.solve_starting_state(path, map, proxy_i))
        } ||
        *block_type == Block::AND && {
            !block.inputs.iter().any(|proxy_i|!self.solve_starting_state(path, map, proxy_i))
        } ||
        *block_type == Block::OR || *block_type == Block::Output && {
            block.inputs.iter().any(|proxy_i|self.solve_starting_state(path,map,proxy_i))
        } ||
        *block_type == Block::XOR && {
            // solve every input starting state, with true / false accumulators, 
            block.inputs.iter().filter(|proxy_i|self.solve_starting_state(path, map, proxy_i)).count() % 2 != 0
        } ||
        *block_type == Block::XNOR && {
            block.inputs.iter().filter(|proxy_i|self.solve_starting_state(path, map, proxy_i)).count() % 2 == 0
        } ||
        // resolve if is input and bit index of input value is true
        *block_type == Block::Input && 
        {
            let (block_map,value)= self.input_hash.get(self.input_label_hash.get(&block_proxy).expect("invalid input data")).expect("invalid input data"); 
            match value {
                BusValue::Bit(b) => *b,
                BusValue::U8(v) => {
                    let bit_index = block_map.iter().position(|&proxy_i|proxy_i == *block_proxy).expect("invalid input data");
                    (v>>bit_index) &1 != 0
                }
                BusValue::U16(v) => {
                    let bit_index = block_map.iter().position(|&proxy_i|proxy_i == *block_proxy).expect("invalid input data");
                    (v>>bit_index) &1 != 0
                }

            }
        };
        path.pop();
        map.insert(*block_proxy,result);
        result
    }
    pub fn create_cm2(self) -> String {
        // id,state,x,y,z,?blockproxy,blockproxy??
        let out_size = self.blocks.len() * 15; // estimate size of output string

        let mut buf = String::with_capacity(out_size);

        let mut solving_path:Vec<BlockProxy> = Vec::new();
        let mut solving_hash:HashMap<BlockProxy,bool> = HashMap::new();

        self.blocks.iter().enumerate().for_each(|(block_i, block)| {
            // &BlockData

            // compute starting state differently
            let block_type = &block.r#type;
            buf.push_str(match block_type {
                Block::NOR => "0,",
                Block::AND => "1,",
                Block::OR => "2,",
                Block::XOR => "3,",
                Block::XNOR => "11,",
                Block::Output => "15,", // what to do here
                Block::Input => "5,",
            });
            buf.push_str(if self.solve_starting_state(&mut solving_path,&mut solving_hash,&BlockProxy::new(block_i)) {
                "1,"
            } else {
                "0,"
            });
            buf.push_str(&format!("{},{},{},",block.position.0,block.position.1,block.position.2));
            buf.push_str(";");
        });
        buf.pop(); // TODO: gate this
        buf.push('?');
        self.blocks.iter().enumerate().for_each(|(block_proxy,block)|{
            block.inputs.iter().for_each(|input_proxy|{
                buf.push_str(&format!("{},{};",1 + input_proxy.value(),1 + block_proxy)); // 1 + n
                                                                                      // cm2 start index is 0
            });
        });
        buf.pop(); //TODO: gate this aswell
        buf.push_str("??");
        buf
    }
    pub fn create_sim(self) -> Simulation<'a> {
        let size: usize = self.blocks.len().next_multiple_of(8);
        let starting_operations: Vec<u8> = self.blocks.iter().map(|b| b.r#type.to_op()).collect();

        let input_table: Vec<Vec<usize>> = self
            .blocks
            .iter()
            .map(|b| b.inputs.iter().map(|v| v.value()).collect())
            .collect();
        // self.input_map
        //     .iter()
        //     .for_each(|(block_map, label, bus_kind)| match bus_kind {
        //         BusKind::Bool => {},
        //         BusKind::U8 => {},
        //     });

        let mut solver_path:Vec<BlockProxy> = Vec::new();
        let mut solver_map:HashMap<BlockProxy,bool> = HashMap::new();


        let starting_state: Vec<u8> = self.blocks.chunks(8).enumerate().map(|(chunk_i,chunk)| {
            chunk.iter().enumerate().fold(0u8, |byte, (local_i, _)| {
                let block_i = chunk_i * 8 + local_i;
                if self.solve_starting_state(&mut solver_path, &mut solver_map, &BlockProxy::new(block_i)) {
                    byte | (1u8 << local_i)
                } else {
                    byte
                }
            })
        }).collect();
        
        // let starting_state: Vec<u8> = self
        //     .blocks
        //     .chunks(8)
        //     .enumerate()
        //     .map(|(chunk_index, chunk)| {
        //         chunk.iter().enumerate().fold(0u8, |byte, (j, block)| {
        //             let i = chunk_index * 8 + j;
        //             if match block.r#type {
        //                 Block::NOR => !self.blocks[i]
        //                     .inputs
        //                     .iter()
        //                     .filter(|input| self.blocks[input.value()].r#type == Block::Input)
        //                     .any(|input| {
        //                         self.input_label_hash
        //                             .get(input)
        //                             .and_then(|label| {
        //                                 let (_, value) = self.input_hash.get(label).unwrap();
        //                                 Some(match value {
        //                                     BusValue::Bool(b) => *b,
        //                                     BusValue::U8(v) => *v != 0,
        //                                   })
        //                             })
        //                             .unwrap_or(false)
        //                     }),
        //                 Block::Input => self
        //                     .input_label_hash
        //                     .get(&BlockProxy::new(i))
        //                     .and_then(|label| {
        //                         if let Some((block_map, bus_value)) = self.input_hash.get(label) {
        //                             let bit_index = block_map
        //                                 .iter()
        //                                 .position(|b| b.value() == i)
        //                                 .expect("somethings not right");
        //                             Some(match bus_value {
        //                                 BusValue::Bool(b) => *b,
        //                                 BusValue::U8(v) => {
        //                                     let v = ((v >> bit_index) & 1) != 0;
        //                                     // println!("{:?},{:?}", bit_index, v);
        //                                     v
        //                                 } // get bool at index
        //                                   // block_map.position(i)
        //                             })
        //                         } else {
        //                             None
        //                         }
        //                         // self.input_map
        //                         //     .iter()
        //                         //     .find(|(_, input_label, _)| label == input_label)
        //                     })
        //                     .unwrap_or(false),
        //                 _ => false,
        //             } {
        //                 byte | (1u8 << j)
        //             } else {
        //                 byte
        //             }
        //         })
        //     })
        //     .collect();

        Simulation::new(&SimulationDescriptor {
            size: size,
            starting_state: starting_state,
            starting_operations: starting_operations,
            input_hash: self.input_hash,
            input_label_hash: self.input_label_hash,
            starting_input_table: input_table,
            output_hash: self.output_hash,
            output_label_hash: self.output_label_hash,
        })
    }
}
