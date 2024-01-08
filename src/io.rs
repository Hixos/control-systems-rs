use anyhow::{anyhow, Result};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
};

#[derive(Debug, Clone)]
pub struct AnySignal {
    value: Rc<RefCell<dyn Any>>, // Option<T>
    name: Option<String>,
    signal_type_id: TypeId,
    signal_type_name: &'static str,
}

impl AnySignal {
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn signal_type_id(&self) -> TypeId {
        self.signal_type_id
    }

    pub fn signal_type_name(&self) -> &str {
        self.signal_type_name
    }
}

impl AnySignal {
    pub(crate) fn new<T: 'static>() -> Self {
        AnySignal {
            value: Rc::new(RefCell::new(Option::<T>::None)),
            name: None,
            signal_type_id: TypeId::of::<T>(),
            signal_type_name: std::any::type_name::<T>(),
        }
    }

    pub(crate) fn try_get<T: Clone + 'static>(&self) -> Result<Option<T>> {
        self.value
            .borrow()
            .downcast_ref::<Option<T>>()
            .ok_or(anyhow!(
                "Trying to get value of type '{}', but signal has is a '{}'",
                std::any::type_name::<T>(),
                self.signal_type_name
            ))
            .map(|v| v.clone())
    }

    pub(crate) fn get<T: Clone + 'static>(&self) -> Option<T> {
        self.try_get().unwrap()
    }

    pub(crate) fn try_set<T: 'static>(&self, value: T) -> Result<()> {
        let mut v = self.value.borrow_mut();
        *v.downcast_mut::<Option<T>>().ok_or(anyhow!(
            "Trying to set value of type '{}', but signal is a '{}'",
            std::any::type_name::<T>(),
            self.signal_type_name
        ))? = Some(value);
        Ok(())
    }

    pub(crate) fn set<T: 'static>(&self, value: T) {
        self.try_set(value).unwrap();
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }
}


#[derive(Debug, Default, Clone)]
pub struct Input<T> {
    phantom: PhantomData<T>,
    signal: Option<AnySignal>,
}

impl<T> Input<T>
where
    T: Clone + 'static,
{
    pub fn get(&self) -> T {
        self.signal.as_ref().unwrap().get::<T>().unwrap()
    }
}

impl<T> Input<T>
where
    T: 'static,
{
    pub fn connect(&mut self, signal: &AnySignal) -> Result<()> {
        debug_assert!(self.signal.is_none(), "Signal is already connected!");

        if signal.signal_type_id() != TypeId::of::<T>() {
            return Err(anyhow!(
                "Cannot connect signal of type '{}' to Input of type '{}'",
                signal.signal_type_name,
                std::any::type_name::<T>()
            ));
        }

        self.signal = Some(signal.clone());
        Ok(())
    }
    
}

impl <T> Input<T> {
    pub fn get_signal(&self) -> &Option<AnySignal> {
        &self.signal
    }

    pub fn get_signal_mut(&mut self) -> &mut Option<AnySignal> {
        &mut self.signal
    }

    pub fn signal_name(&self) -> String {
        self.signal.as_ref().unwrap().name.as_ref().unwrap().clone()
    }
}

#[derive(Debug)]
pub struct Output<T> {
    phantom: PhantomData<T>,
    signal: AnySignal,
}

impl<T: 'static> Default for Output<T> {
    fn default() -> Self {
        Output {
            phantom: PhantomData,
            signal: AnySignal::new::<T>(),
        }
    }
}

impl<T> Output<T>
where
    T: 'static,
{
    pub fn set(&mut self, value: T) {
        self.signal.set(value)
    }
}

impl<T> Output<T>
{
    pub fn get_signal(&self) -> &AnySignal {
        &self.signal
    }

    pub fn get_signal_mut(&mut self) -> &mut AnySignal {
        &mut self.signal
    }

    pub fn signal_name(&self) -> String {
        self.signal.name.as_ref().unwrap().clone()
    }
}
