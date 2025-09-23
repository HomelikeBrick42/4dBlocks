pub mod state;
pub mod ui;

use crate::state::State;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

fn main() -> Result<(), winit::error::EventLoopError> {
    struct WindowState {
        window: Arc<Window>,
        surface_config: wgpu::SurfaceConfiguration,
        surface: wgpu::Surface<'static>,
    }

    struct App {
        last_time: Option<Instant>,
        dt: Duration,

        instance: wgpu::Instance,
        device: wgpu::Device,
        queue: wgpu::Queue,

        state: State,
        window_state: Option<WindowState>,
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            self.suspended(event_loop);

            let window = Arc::new(
                event_loop
                    .create_window(WindowAttributes::default().with_title("4d Blocks"))
                    .expect("window should be created"),
            );

            let mut surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: 0,
                height: 0,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
            };
            let surface = self
                .instance
                .create_surface(window.clone())
                .expect("surface should be created");

            PhysicalSize {
                width: surface_config.width,
                height: surface_config.height,
            } = window.inner_size();
            Self::recreate_surface(&self.device, &surface, &surface_config);
            self.state
                .surface_resized(surface_config.width, surface_config.height);

            self.window_state = Some(WindowState {
                window,
                surface_config,
                surface,
            });
        }

        fn suspended(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
            self.last_time = None;
            self.window_state = None;
        }

        fn new_events(
            &mut self,
            #[expect(unused)] event_loop: &ActiveEventLoop,
            #[expect(unused)] cause: StartCause,
        ) {
            let time = Instant::now();
            self.dt = time - self.last_time.unwrap_or(time);
            self.last_time = Some(time);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
            let WindowState {
                window,
                surface_config,
                surface,
            } = self
                .window_state
                .as_mut()
                .expect("window should have been created if there are window events");

            assert_eq!(window.id(), window_id);
            match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),

                WindowEvent::Resized(_) => {
                    PhysicalSize {
                        width: surface_config.width,
                        height: surface_config.height,
                    } = window.inner_size();
                    Self::recreate_surface(&self.device, surface, surface_config);
                    self.state
                        .surface_resized(surface_config.width, surface_config.height);

                    self.render();
                }

                _ => {}
            }
        }

        fn about_to_wait(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
            self.state.update(self.dt.as_secs_f32());
            self.render();
        }
    }

    impl App {
        fn recreate_surface(
            device: &wgpu::Device,
            surface: &wgpu::Surface,
            surface_config: &wgpu::SurfaceConfiguration,
        ) {
            if surface_config.width == 0 || surface_config.height == 0 {
                return;
            }
            surface.configure(device, surface_config);
        }

        fn render(&mut self) {
            let Some(WindowState {
                window,
                surface_config,
                surface,
            }) = &mut self.window_state
            else {
                return;
            };

            if surface_config.width == 0 || surface_config.height == 0 {
                return;
            }

            let surface_texture = match surface.get_current_texture() {
                Ok(surface_texture) => surface_texture,
                Err(wgpu::SurfaceError::Timeout) => return,
                Err(wgpu::SurfaceError::Outdated) => {
                    PhysicalSize {
                        width: surface_config.width,
                        height: surface_config.height,
                    } = window.inner_size();
                    Self::recreate_surface(&self.device, surface, surface_config);
                    self.state
                        .surface_resized(surface_config.width, surface_config.height);
                    return;
                }
                Err(wgpu::SurfaceError::Lost) => {
                    Self::recreate_surface(&self.device, surface, surface_config);
                    return;
                }
                Err(e) => panic!("{e}"),
            };

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            let render_callback = self.state.render(&self.device, &self.queue, &mut encoder);

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_texture.texture.create_view(&Default::default()),
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.0,
                                g: 0.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_callback(&mut render_pass);
            }

            self.queue.submit(std::iter::once(encoder.finish()));

            let suboptimal = surface_texture.suboptimal;
            surface_texture.present();
            if suboptimal {
                Self::recreate_surface(&self.device, surface, surface_config);
            }
        }
    }

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all().with_env(),
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        ..Default::default()
    });
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("adapter should be created");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("device and queue should be created");

        (device, queue)
    });

    let state = State::new(&device);

    let mut app = App {
        last_time: None,
        dt: Duration::ZERO,

        instance,
        device,
        queue,

        state,
        window_state: None,
    };

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)
}
