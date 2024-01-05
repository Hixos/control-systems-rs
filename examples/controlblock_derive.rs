use controlblock_derive::ControlBlock;
use control_system::{InputConnector, OutputConnector, Interconnector, StepInfo};
use anyhow::Result;

pub trait ControlBlock {
    fn name(&self) -> String;

    fn register_outputs(&mut self, interconnector: &mut Interconnector) -> Result<()>;

    fn register_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()>;
}

pub trait DiscreteTimeBlock : ControlBlock {
    fn step(&mut self, k: StepInfo) -> Result<()>;

    fn delay(&self) -> usize {
        0usize
    }
}

#[derive(ControlBlock)]
struct DerivedBlock {
    #[controlblock(block_name)]
    name: String,   

    #[controlblock(input)]
    u1: InputConnector<f64>,
    #[controlblock(input, name="u_name_2")]
    u2: InputConnector<f64>,

    #[controlblock(output)]
    y: OutputConnector<f64>,
}

impl DerivedBlock {
   
}

impl ControlBlock for DerivedBlock {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn register_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.u1)?;
        interconnector.register_input(&mut self.u2)?;

        Ok(())
    }
    
    fn register_outputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_output(&mut self.y)?;
        Ok(())
    }
}

// impl DiscreteTimeBlock for DerivedBlock {
//     fn step(&mut self, k: StepInfo) -> Result<()> {
//         self.y.output(self.u1.input() + self.u2.input());
//         Ok(())
//     }
// }

fn main(){
    // let mut block = DerivedBlock::new();
}