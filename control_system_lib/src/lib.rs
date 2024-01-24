mod controlblock;
mod controlsystem;
mod parameters;

pub mod io;
pub mod numeric;
use std::error::Error;

pub use control_system_derive::BlockIO;

pub use controlblock::{Block, BlockIO, StepInfo, StepResult};
pub use controlsystem::{ControlSystem, ControlSystemBuilder, ControlSystemParameters};
pub use parameters::{ParameterStore, ParameterStoreError};

use thiserror::Error;

pub type Result<T, E = ControlSystemError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum ControlSystemError {
    #[error("A block named '{0}' is already present in the control system")]
    DuplicateBlockName(String),

    #[error("Could not connect port {port} of block '{blockname}': No signal named '{signal}'")]
    UnknownSignal {
        port: String,
        signal: String,
        blockname: String,
    },

    #[error("No port named '{port}' in block '{blockname}'")]
    UnknownPort { port: String, blockname: String },

    #[error("Control system presents a cycle containing node '{0}'")]
    CycleDetected(String),

    #[error("Cannot connect output '{port}' of block '{blockname}' to signal '{signal}': The signal is already connected to another output.")]
    MultipleProducers {
        port: String,
        signal: String,
        blockname: String,
    },

    #[error("Ports {ports:?} in block '{blockname}' have not been connected")]
    UnconnectedPorts {
        ports: Vec<String>,
        blockname: String,
    },

    #[error("Expected signal '{signal}' to be a '{typename}', but is a '{signal_typename}'")]
    TypeError {
        signal: String,
        typename: String,
        signal_typename: String,
    },

    #[error(transparent)]
    ParameterError {
        #[from]
        source: ParameterStoreError,
    },

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl ControlSystemError {
    pub fn from_boxed<E: Error + Send + Sync + 'static>(e: E) -> Self {
        ControlSystemError::Other(Box::new(e) as Box<dyn Error + Send + Sync + 'static>)
    }
}