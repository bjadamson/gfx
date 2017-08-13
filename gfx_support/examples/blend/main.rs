// Copyright 2017 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[deny(dead_code)]
#[macro_use]
extern crate gfx;
extern crate gfx_support;
extern crate winit;

use gfx_support::{Application, BackbufferView, ColorFormat};
use gfx::Bundle;
use gfx::GraphicsPoolExt;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Locals {
        blend: i32 = "u_Blend",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        lena: gfx::TextureSampler<[f32; 4]> = "t_Lena",
        tint: gfx::TextureSampler<[f32; 4]> = "t_Tint",
        blend: gfx::Global<i32> = "i_Blend",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

impl Vertex {
    fn new(p: [f32; 2], u: [f32; 2]) -> Vertex {
        Vertex { pos: p, uv: u }
    }
}

const BLENDS: [&'static str; 9] = ["Screen",
                                   "Dodge",
                                   "Burn",
                                   "Overlay",
                                   "Multiply",
                                   "Add",
                                   "Divide",
                                   "Grain Extract",
                                   "Grain Merge"];

struct App<B: gfx::Backend> {
    bundle: Bundle<B, pipe::Data<B::Resources>>,
    id: u8,
    views: Vec<BackbufferView<B::Resources>>,
}

impl<B: gfx::Backend> Application<B> for App<B> {
    fn new(device: &mut B::Device,
           _: &mut gfx::queue::GraphicsQueue<B>,
           backend: gfx_support::shade::Backend,
           window_targets: gfx_support::WindowTargets<B::Resources>)
           -> Self {
        use gfx::traits::DeviceExt;

        let vs = gfx_support::shade::Source {
            glsl_120: include_bytes!("shader/blend_120.glslv"),
            glsl_150: include_bytes!("shader/blend_150.glslv"),
            hlsl_40: include_bytes!("data/vertex.fx"),
            ..gfx_support::shade::Source::empty()
        };
        let ps = gfx_support::shade::Source {
            glsl_120: include_bytes!("shader/blend_120.glslf"),
            glsl_150: include_bytes!("shader/blend_150.glslf"),
            hlsl_40: include_bytes!("data/pixel.fx"),
            ..gfx_support::shade::Source::empty()
        };

        // fullscreen quad
        let vertex_data = [Vertex::new([-1.0, -1.0], [0.0, 1.0]),
                           Vertex::new([1.0, -1.0], [1.0, 1.0]),
                           Vertex::new([1.0, 1.0], [1.0, 0.0]),

                           Vertex::new([-1.0, -1.0], [0.0, 1.0]),
                           Vertex::new([1.0, 1.0], [1.0, 0.0]),
                           Vertex::new([-1.0, 1.0], [0.0, 0.0])];
        let (vbuf, slice) = device.create_vertex_buffer_with_slice(&vertex_data, ());

        let lena_texture = gfx_support::load_texture(device, &include_bytes!("image/lena.png")[..])
            .unwrap();
        let tint_texture = gfx_support::load_texture(device, &include_bytes!("image/tint.png")[..])
            .unwrap();
        let sampler = device.create_sampler_linear();

        let pso = device.create_pipeline_simple(vs.select(backend).unwrap(),
                                    ps.select(backend).unwrap(),
                                    pipe::new())
            .unwrap();

        // we pass a integer to our shader to show what blending function we want
        // it to use. normally you'd have a shader program per technique, but for
        // the sake of simplicity we'll just branch on it inside the shader.

        // each index correspond to a conditional branch inside the shader
        println!("Using '{}' blend equation", BLENDS[0]);
        let cbuf = device.create_constant_buffer(1);

        let data = pipe::Data {
            vbuf: vbuf,
            lena: (lena_texture, sampler.clone()),
            tint: (tint_texture, sampler),
            blend: 0,
            locals: cbuf,
            out: window_targets.views[0].0.clone(),
        };

        App {
            bundle: Bundle::new(slice, pso, data),
            id: 0,
            views: window_targets.views,
        }
    }

    fn render(&mut self,
              (frame, sync): (gfx::Frame, &gfx_support::SyncPrimitives<B::Resources>),
              pool: &mut gfx::GraphicsCommandPool<B>,
              queue: &mut gfx::queue::GraphicsQueue<B>) {
        let (cur_color, _) = self.views[frame.id()].clone();
        self.bundle.data.out = cur_color;

        let mut encoder = pool.acquire_graphics_encoder();
        self.bundle.data.blend = (self.id as i32).into();
        let locals = Locals { blend: self.id as i32 };
        encoder.update_constant_buffer(&self.bundle.data.locals, &locals);
        encoder.clear(&self.bundle.data.out, [0.0; 4]);
        self.bundle.encode(&mut encoder);
        encoder.synced_flush(queue, &[&sync.rendering], &[], Some(&sync.frame_fence))
            .expect("Could not flush encoder");
    }

    fn on(&mut self, event: winit::WindowEvent) {
        if let winit::WindowEvent::KeyboardInput {
                input: winit::KeyboardInput {
                    state: winit::ElementState::Pressed,
                    virtual_keycode: Some(winit::VirtualKeyCode::B),
                    ..
                },
                .. } = event {
            self.id += 1;
            if self.id as usize >= BLENDS.len() {
                self.id = 0;
            }
            println!("Using '{}' blend equation", BLENDS[self.id as usize]);
        }
    }

    fn on_resize(&mut self, window_targets: gfx_support::WindowTargets<B::Resources>) {
        self.views = window_targets.views;
    }
}

pub fn main() {
    App::launch_simple("Blending example");
}
