use std::{
    path::Path,
    sync::mpsc::{channel, Sender},
    thread::spawn,
};

use anyhow::Result;
use control_system::{
    io::{Input, Output},
    numeric::ode::{ODESolver, RungeKutta4},
    Block, ControlSystemParameters, ParameterStore, StepInfo, StepResult,
};
use control_system::{BlockIO, ControlSystemBuilder};
use control_system_blocks::{Constant, ConstantParams};
use control_system_derive::BlockIO;
use control_system_plotter::add_plotter;
use nalgebra::Vector2;
use rust_data_inspector::datainspector::DataInspector;
use rust_data_inspector_signals::PlotSignals;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CartParams {
    mass: f64,
    pos0: f64,
    vel0: f64,
}

#[derive(BlockIO)]
struct Cart {
    #[blockio(block_name)]
    name: String,

    #[blockio(input)]
    u_force: Input<f64>,

    #[blockio(output)]
    y_pos: Output<f64>,
    #[blockio(output)]
    y_vel: Output<f64>,
    #[blockio(output)]
    y_acc: Output<f64>,

    params: CartParams,
    state: Vector2<f64>,
}

impl Block for Cart {
    fn step(&mut self, k: StepInfo) -> control_system::Result<StepResult> {
        let acc = self.u_force.get() / self.params.mass;

        let odefun = |_, x: Vector2<f64>| Vector2::new(x[1], acc);

        // Propagate state
        self.state = RungeKutta4::solve(odefun, k.t, k.dt, self.state);

        // Assign outputs
        self.y_pos.set(self.state[0]);
        self.y_vel.set(self.state[1]);
        self.y_acc.set(acc);

        Ok(StepResult::Continue)
    }
}

impl Cart {
    fn new(params: CartParams) -> Self {
        Cart {
            name: "cart".to_string(),
            state: Vector2::new(params.pos0, params.vel0),
            params,
            u_force: Input::default(),
            y_pos: Output::default(),
            y_vel: Output::default(),
            y_acc: Output::default(),
        }
    }

    fn from_store(store: &mut ParameterStore, default_params: CartParams) -> Result<Self> {
        let params = store.get_block_params("cart", default_params)?;

        Ok(Self::new(params))
    }
}

fn main() -> Result<()> {
    let (signals_snd, signals_rcv) = channel();

    spawn(move || {
        run_control_system(signals_snd).unwrap();
    });

    let signals = signals_rcv.recv().expect("Error receiving signals");

    DataInspector::run_native("Cart control system", signals).expect("Error");
    Ok(())
}

fn run_control_system(signals_snd: Sender<PlotSignals>) -> Result<()> {
    let mut store = ParameterStore::new(Path::new("cart.toml"), "cart")?;

    let mut signals = PlotSignals::default();

    let cart = Cart::from_store(
        &mut store,
        CartParams {
            mass: 1.0,
            pos0: 0.0,
            vel0: 0.0,
        },
    )?;

    let constant = Constant::from_store("force", &mut store, ConstantParams { c: 100.0 })?;

    let mut builder = ControlSystemBuilder::default();

    builder.add_block(
        cart,
        &[("u_force", "/force")],
        &[
            ("y_pos", "/cart/pos"),
            ("y_vel", "/cart/vel"),
            ("y_acc", "/cart/acc"),
        ],
    )?;

    builder.add_block(constant, &[], &[("y", "/force")])?;

    add_plotter::<f64>("/cart/pos", &mut builder, &mut signals)?;
    add_plotter::<f64>("/cart/vel", &mut builder, &mut signals)?;
    add_plotter::<f64>("/cart/acc", &mut builder, &mut signals)?;
    add_plotter::<f64>("/force", &mut builder, &mut signals)?;

    // Signals are properly populated. Send them to be used by the GUI
    signals_snd
        .send(signals)
        .expect("Could not send signals to GUI");

    let mut cs =
        builder.build_from_store("cart", &mut store, ControlSystemParameters::new(0.01))?;

    store.save()?;

    for _ in 0..1000 {
        if cs.step()? == StepResult::Stop {
            break;
        };
    }

    Ok(())
}
