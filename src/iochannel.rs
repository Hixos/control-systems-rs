use std::{cell::Cell, rc::Rc};


#[derive(Clone, Copy)]
struct IOInner<T> {
    data: Option<T>,
}

pub struct IOChannel<T> {
    inner_data: Rc<Cell<IOInner<T>>>,
}

impl<T> IOChannel<T> {
    pub fn new_input_receiver(&self) -> InputReceiver<T> {
        InputReceiver {
            inner_data: self.inner_data.clone(),
        }
    }
}

pub struct OutputSender<T> {
    inner_data: Rc<Cell<IOInner<T>>>,
}

impl<T: Copy> OutputSender<T> {
    pub fn output(&self, val: T) {
        self.inner_data.set(IOInner { data: Some(val) })
    }
}

pub struct InputReceiver<T> {
    inner_data: Rc<Cell<IOInner<T>>>,
}

impl<T: Copy> InputReceiver<T> {
    pub fn input(&self) -> Option<T> {
        self.inner_data.get().data
    }
}

pub fn io_channel<T>() -> (IOChannel<T>, OutputSender<T>) {
    let inner_data: Rc<Cell<IOInner<T>>> = Rc::new(Cell::new(IOInner { data: None }));
    (
        IOChannel {
            inner_data: inner_data.clone(),
        },
        OutputSender {
            inner_data: inner_data.clone(),
        },
    )
}