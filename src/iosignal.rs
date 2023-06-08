use std::{cell::Cell, rc::Rc};


#[derive(Clone, Copy)]
struct IOInner<T> {
    data: Option<T>,
}

pub(crate) struct IOSignal<T> {
    inner_data: Rc<Cell<IOInner<T>>>,
}

impl<T> IOSignal<T> {
    pub(crate) fn new_input_receiver(&self) -> InputReceiver<T> {
        InputReceiver {
            inner_data: self.inner_data.clone(),
        }
    }
}

pub(crate) struct OutputSender<T> {
    inner_data: Rc<Cell<IOInner<T>>>,
}

impl<T: Copy> OutputSender<T> {
    pub(crate) fn output(&self, val: T) {
        self.inner_data.set(IOInner { data: Some(val) })
    }
}

pub(crate) struct InputReceiver<T> {
    inner_data: Rc<Cell<IOInner<T>>>,
}

impl<T: Copy> InputReceiver<T> {
    pub(crate) fn input(&self) -> Option<T> {
        self.inner_data.get().data
    }
}

pub(crate) fn io_signal<T>() -> (IOSignal<T>, OutputSender<T>) {
    let inner_data: Rc<Cell<IOInner<T>>> = Rc::new(Cell::new(IOInner { data: None }));
    (
        IOSignal {
            inner_data: inner_data.clone(),
        },
        OutputSender {
            inner_data: inner_data.clone(),
        },
    )
}