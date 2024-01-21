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
use control_system_blocks::{
    siso::{PIDParams, PID},
    Add, Constant, Delay,
};
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

    let handle = spawn(move || run_control_system(signals_snd) );//expect("Error running control system"));

    if let Ok(signals) = signals_rcv.recv() {
        DataInspector::run_native("Cart control system", signals).expect("Error running GUI");
    }

    let cs_result = handle.join().expect("Thread panicked");
    if let Err(err) = cs_result {
        println!("{:?}", err);
    }
    Ok(())
}

fn run_control_system(signals_snd: Sender<PlotSignals>) -> Result<()> {
    let mut store = ParameterStore::new(Path::new("cart.toml"), "cart")?;

    let mut builder = ControlSystemBuilder::default();

    // Cart
    builder.add_block(
        Cart::from_store(
            &mut store,
            CartParams {
                mass: 1.0,
                pos0: 0.0,
                vel0: 0.0,
            },
        )?,
        &[("u_force", "/force")],
        &[
            ("y_pos", "/cart/pos"),
            ("y_vel", "/cart/vel"),
            ("y_acc", "/cart/acc"),
        ],
    )?;

    // ** Inner loop **
    // Inputs:
    // - /ref/vel: Velocity reference
    // Outputs:
    // - /force: Commanded force on the cart

    builder.add_block(
        Delay::from_store("vel_delay", &mut store, [0.0].into())?,
        &[("u", "/cart/vel")],
        &[("y", "/cart/vel_delayed")],
    )?;

    builder.add_block(
        Add::<f64, 2>::new("vel_err", [1.0, -1.0].into()),
        &[("u1", "/ref/vel"), ("u2", "/cart/vel_delayed")],
        &[("y", "/err/vel")],
    )?;

    builder.add_block(
        PID::from_store(
            "pid_vel",
            &mut store,
            PIDParams {
                kp: 4.0,
                ..Default::default()
            },
        )?,
        &[("u", "/err/vel")],
        &[("y", "/force")],
    )?;

    // ** Outer Loop **
    // Inputs:
    // - /ref/pos: Position reference
    // Outputs:
    // - /ref/vel: Velocity reference

    builder.add_block(
        Delay::from_store("pos_delay", &mut store, [0.0].into())?,
        &[("u", "/cart/pos")],
        &[("y", "/cart/pos_delayed")],
    )?;

    builder.add_block(
        Add::<f64, 2>::new("pos_err", [1.0, -1.0].into()),
        &[("u1", "/ref/pos"), ("u2", "/cart/pos_delayed")],
        &[("y", "/err/pos")],
    )?;

    builder.add_block(
        PID::from_store(
            "pid_pos",
            &mut store,
            PIDParams {
                kp: 1.0,
                ..Default::default()
            },
        )?,
        &[("u", "/err/pos")],
        &[("y", "/ref/vel")],
    )?;

    // Position reference

    builder.add_block(
        Constant::from_store("pos_ref", &mut store, 15.0.into())?,
        &[],
        &[("y", "/ref/pos")],
    )?;

    // Plotters
    let mut signals = PlotSignals::default();
    add_plotter::<f64>("/cart/pos", &mut builder, &mut signals)?;
    add_plotter::<f64>("/cart/vel", &mut builder, &mut signals)?;
    add_plotter::<f64>("/cart/acc", &mut builder, &mut signals)?;
    add_plotter::<f64>("/force", &mut builder, &mut signals)?;
    add_plotter::<f64>("/ref/pos", &mut builder, &mut signals)?;
    add_plotter::<f64>("/ref/vel", &mut builder, &mut signals)?;
    add_plotter::<f64>("/err/pos", &mut builder, &mut signals)?;
    add_plotter::<f64>("/err/vel", &mut builder, &mut signals)?;

    // Build the control system
    let mut cs =
        builder.build_from_store("cart", &mut store, ControlSystemParameters::new(0.01))?;

    store.save()?;

    // Signals are properly populated. Send them to be used by the GUI
    signals_snd
        .send(signals)
        .expect("Could not send signals to GUI");

    // Execute
    for _ in 0..1000 {
        if cs.step()? == StepResult::Stop {
            break;
        };
    }

    Ok(())
}
