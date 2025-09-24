mod app;
pub mod state;
pub mod ui;
pub mod camera;
pub mod ray_tracing;

pub use app::Input;

fn main() -> Result<(), winit::error::EventLoopError> {
    app::main()
}
