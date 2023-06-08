use std::marker::PhantomData;

use crate::{ControlBlock, InputConnector, Interconnector, StepInfo};
use anyhow::Result;

pub trait Prober<T> {
    fn probe(&self, signal: &str, v: Option<T>, k: StepInfo);
}

pub(crate) struct FnProber<T, F>
where
    F: Fn(&str, Option<T>, StepInfo),
{
    f: F,
    phantom: PhantomData<T>,
}

impl<T, F> FnProber<T, F>
where
    F: Fn(&str, Option<T>, StepInfo),
{
    pub(crate) fn new(f: F) -> Self {
        FnProber {
            f,
            phantom: PhantomData::default(),
        }
    }
}

impl<T, F> Prober<T> for FnProber<T, F>
where
    F: Fn(&str, Option<T>, StepInfo),
{
    fn probe(&self, signal: &str, v: Option<T>, k: StepInfo) {
        (self.f)(signal, v, k);
    }
}

pub(crate) struct Probe<T, P>
where
    P: Prober<T>,
{
    name: String,
    u: InputConnector<T>,
    p: P,
}

impl<T: Copy, P> Probe<T, P>
where
    P: Prober<T>,
{
    pub(crate) fn new(block_name: &str, in_name: &str, p: P) -> Self {
        Probe {
            name: block_name.to_string(),
            u: InputConnector::new(in_name),
            p: p,
        }
    }
}

impl<T: Copy + 'static, P> ControlBlock for Probe<T, P>
where
    P: Prober<T>,
{
    fn register_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.u)?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_outputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn step(&mut self, k: StepInfo) -> Result<()> {
        self.p.probe(self.u.name(), self.u.input(), k);
        Ok(())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
