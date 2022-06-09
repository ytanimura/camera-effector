use clap::{Arg, Command};
use glium::{
    glutin, implement_vertex, index::PrimitiveType, program, texture::RawImage2d, uniform, Display,
    IndexBuffer, Surface, Texture2d, VertexBuffer,
};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use nokhwa::{nokhwa_initialize, query_devices, Camera, CaptureAPIBackend, FrameFormat};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
}

/// serialized `FrameFormat`
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SerializedFrameFormat {
    MJPEG,
    YUYV,
}

impl From<SerializedFrameFormat> for FrameFormat {
    fn from(x: SerializedFrameFormat) -> Self {
        match x {
            SerializedFrameFormat::MJPEG => Self::MJPEG,
            SerializedFrameFormat::YUYV => Self::YUYV,
        }
    }
}

/// abbreviation of `CaptureAPIBackend`s.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum AbbreviationCaptureAPIBackend {
    /// Auto
    AUTO,
    /// UniversalVideoClass
    UVC,
    /// GStreamer
    GST,
    /// Video4Linux
    V4L,
    /// MediaFoundation
    MSMF,
    /// AVFoundation
    AVF,
    /// OpenCV
    OPENCV,
}

impl From<AbbreviationCaptureAPIBackend> for CaptureAPIBackend {
    fn from(x: AbbreviationCaptureAPIBackend) -> Self {
        use AbbreviationCaptureAPIBackend::*;
        match x {
            AUTO => CaptureAPIBackend::Auto,
            UVC => CaptureAPIBackend::UniversalVideoClass,
            GST => CaptureAPIBackend::GStreamer,
            V4L => CaptureAPIBackend::GStreamer,
            MSMF => CaptureAPIBackend::MediaFoundation,
            OPENCV => CaptureAPIBackend::OpenCv,
            AVF => CaptureAPIBackend::AVFoundation,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CameraBuilder {
    pub resolution: (u32, u32),
    pub format: SerializedFrameFormat,
    pub frame_rate: u32,
    pub backend: AbbreviationCaptureAPIBackend,
    pub index: usize,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            resolution: (640, 480),
            format: SerializedFrameFormat::MJPEG,
            frame_rate: 30,
            backend: AbbreviationCaptureAPIBackend::AUTO,
            index: 0,
        }
    }
}

fn main() {
    let matches = Command::new("nokhwa-example")
        .version("0.1.0")
        .author("l1npengtul <l1npengtul@protonmail.com> and the Nokhwa Contributers")
        .about("Example program using Nokhwa")
        .arg(Arg::new("init").long("init").help(
            "Only configuration file generation and device enumeration. No capturing is performed.",
        ))
        .get_matches();
    let setting: CameraBuilder = match std::fs::read_to_string("camera_setting.json") {
        Ok(json) => serde_json::from_str(&json).unwrap_or_else(|_| {
            eprintln!("Failed to parse json. The default setting is applied.");
            Default::default()
        }),
        Err(_) => {
            let setting = Default::default();
            if std::fs::write(
                "camera_setting.json",
                &serde_json::to_string_pretty(&setting).unwrap(),
            )
            .is_err()
            {
                eprintln!("Failed to create camera_setting.json.");
            };
            setting
        }
    };

    nokhwa_initialize(|_| {});
    let backend_value = setting.backend.into();

    match query_devices(backend_value) {
        Ok(devs) => {
            for (idx, camera) in devs.iter().enumerate() {
                println!("Device at index {}: {}", idx, camera)
            }
        }
        Err(why) => {
            println!("Failed to query: {why}")
        }
    }

    if !matches.is_present("init") {
        let (send, recv) = flume::unbounded();
        // spawn a thread for capture
        std::thread::spawn(move || {
            let mut camera = Camera::new_with(
                setting.index,
                setting.resolution.0,
                setting.resolution.1,
                setting.frame_rate,
                setting.format.into(),
                backend_value,
            )
            .unwrap();

            // open stream
            camera.open_stream().unwrap();
            loop {
                if let Ok(frame) = camera.frame() {
                    let _send = send.send(frame);
                }
            }
        });

        // run glium
        let gl_event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new();
        let context_builder = ContextBuilder::new().with_vsync(true);
        let gl_display = Display::new(window_builder, context_builder, &gl_event_loop).unwrap();

        implement_vertex!(Vertex, position);

        let vert_buffer = VertexBuffer::new(
            &gl_display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                },
            ],
        )
        .unwrap();

        let idx_buf =
            IndexBuffer::new(&gl_display, PrimitiveType::TriangleStrip, &[1_u16, 2, 0, 3]).unwrap();

        let fragment = "#version 300 es\nprecision highp float;uniform sampler2D iChannel0;uniform float iTime;uniform vec3 iResolution;\
out vec4 _color_;void mainImage(out vec4, in vec2);void main() {mainImage(_color_,gl_FragCoord.xy);}".to_string() + include_str!("shader.frag");

        let program = program!(&gl_display,
                300 es => {
                    vertex: "#version 300 es\nin vec2 position;void main(){gl_Position=vec4(position,0,1);}",
                    outputs_srgb: true,
                    fragment: &fragment,
                },
            )
            .unwrap();
        let instant = std::time::Instant::now();

        // run the event loop
        gl_event_loop.run(move |event, _window, ctrl| {
            *ctrl = match event {
                Event::MainEventsCleared => {
                    let frame = recv.recv().unwrap();

                    let frame_size = (frame.width(), frame.height());

                    let raw_data = RawImage2d::from_raw_rgb_reversed(&frame.into_raw(), frame_size);
                    let gl_texture = Texture2d::new(&gl_display, raw_data).unwrap();

                    let res = gl_display.get_framebuffer_dimensions();

                    let uniforms = uniform! {
                        iTime: instant.elapsed().as_secs_f32(),
                        iResolution: [res.0 as f32, res.1 as f32, 1.0],
                        iChannel0: &gl_texture
                    };

                    let mut target = gl_display.draw();
                    target.clear_color(0.0, 0.0, 0.0, 0.0);
                    target
                        .draw(
                            &vert_buffer,
                            &idx_buf,
                            &program,
                            &uniforms,
                            &Default::default(),
                        )
                        .unwrap();
                    target.finish().unwrap();

                    ControlFlow::Poll
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => ControlFlow::Exit,
                _ => ControlFlow::Poll,
            }
        })
    }
}
