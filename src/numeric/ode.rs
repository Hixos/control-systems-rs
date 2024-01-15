use nalgebra::SVector;

pub trait ODESolver<T>
{
    fn solve<const D: usize, F>(f: F, t0: T, dt: T, y0: SVector<T, D>) -> SVector<T, D>
    where
        F: Fn(T, SVector<T, D>) -> SVector<T, D>;
}

pub struct RungeKutta4;

impl ODESolver<f64> for RungeKutta4
{
    fn solve<const D: usize, F>(f: F, t0: f64, dt: f64, y0: SVector<f64, D>) -> SVector<f64, D>
    where
        F: Fn(f64, SVector<f64, D>) -> SVector<f64, D>,
    {
        let hdt = dt;

        let k1 = f(t0, y0);
        let k2 = f(t0 + hdt, y0 + k1 * hdt);
        let k3 = f(t0 + hdt, y0 + k2 * hdt);
        let k4 = f(t0 + dt, y0 + k3 * dt);

        y0 + (k1 + 2f64*k2 + 2f64*k3 + k4) * dt / 6f64
    }
}


pub struct ForwardEuler;

impl ODESolver<f64> for ForwardEuler
{
    fn solve<const D: usize, F>(f: F, t0: f64, dt: f64, y0: SVector<f64, D>) -> SVector<f64, D>
    where
        F: Fn(f64, SVector<f64, D>) -> SVector<f64, D>,
    {
        y0 + f(t0, y0) * dt
    }
}