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

#[macro_use]
extern crate gfx;
extern crate gfx_support;

use std::time::Instant;
use gfx_support::{Application, BackbufferView, ColorFormat};
use gfx::{Bundle, GraphicsPoolExt};

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Locals {
        offsets: [f32; 2] = "u_Offsets",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        flow: gfx::TextureSampler<[f32; 4]> = "t_Flow",
        noise: gfx::TextureSampler<[f32; 4]> = "t_Noise",
        offset0: gfx::Global<f32> = "f_Offset0",
        offset1: gfx::Global<f32> = "f_Offset1",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

impl Vertex {
    fn new(p: [f32; 2], u: [f32; 2]) -> Vertex {
        Vertex { pos: p, uv: u }
    }
}

struct App<B: gfx::Backend> {
    views: Vec<BackbufferView<B::Resources>>,
    bundle: Bundle<B, pipe::Data<B::Resources>>,
    cycles: [f32; 2],
    time_start: Instant,
}

impl<B: gfx::Backend> Application<B> for App<B> {
    fn new(device: &mut B::Device,
           _: &mut gfx::queue::GraphicsQueue<B>,
           backend: gfx_support::shade::Backend,
           window_targets: gfx_support::WindowTargets<B::Resources>)
           -> Self {
        use gfx::traits::DeviceExt;

        let vs = gfx_support::shade::Source {
            glsl_120: include_bytes!("shader/flowmap_120.glslv"),
            glsl_150: include_bytes!("shader/flowmap_150.glslv"),
            hlsl_40: include_bytes!("data/vertex.fx"),
            msl_11: include_bytes!("shader/flowmap_vertex.metal"),
            ..gfx_support::shade::Source::empty()
        };
        let ps = gfx_support::shade::Source {
            glsl_120: include_bytes!("shader/flowmap_120.glslf"),
            glsl_150: include_bytes!("shader/flowmap_150.glslf"),
            hlsl_40: include_bytes!("data/pixel.fx"),
            msl_11: include_bytes!("shader/flowmap_frag.metal"),
            ..gfx_support::shade::Source::empty()
        };

        let vertex_data = [Vertex::new([-1.0, -1.0], [0.0, 0.0]),
                           Vertex::new([1.0, -1.0], [1.0, 0.0]),
                           Vertex::new([1.0, 1.0], [1.0, 1.0]),

                           Vertex::new([-1.0, -1.0], [0.0, 0.0]),
                           Vertex::new([1.0, 1.0], [1.0, 1.0]),
                           Vertex::new([-1.0, 1.0], [0.0, 1.0])];

        let (vbuf, slice) = device.create_vertex_buffer_with_slice(&vertex_data, ());

        let water_texture =
            gfx_support::load_texture(device, &include_bytes!("image/water.png")[..]).unwrap();
        let flow_texture = gfx_support::load_texture(device, &include_bytes!("image/flow.png")[..])
            .unwrap();
        let noise_texture =
            gfx_support::load_texture(device, &include_bytes!("image/noise.png")[..]).unwrap();
        let sampler = device.create_sampler_linear();

        let pso = device.create_pipeline_simple(vs.select(backend).unwrap(),
                                    ps.select(backend).unwrap(),
                                    pipe::new())
            .unwrap();

        let data = pipe::Data {
            vbuf: vbuf,
            color: (water_texture, sampler.clone()),
            flow: (flow_texture, sampler.clone()),
            noise: (noise_texture, sampler.clone()),
            offset0: 0.0,
            offset1: 0.0,
            locals: device.create_constant_buffer(1),
            out: window_targets.views[0].0.clone(),
        };

        App {
            views: window_targets.views,
            bundle: Bundle::new(slice, pso, data),
            cycles: [0.0, 0.5],
            time_start: Instant::now(),
        }
    }

    fn render(&mut self,
              (frame, sync): (gfx::Frame, &gfx_support::SyncPrimitives<B::Resources>),
              pool: &mut gfx::GraphicsCommandPool<B>,
              queue: &mut gfx::queue::GraphicsQueue<B>) {
        let delta = self.time_start.elapsed();
        self.time_start = Instant::now();
        let delta = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1000_000_000.0;

        // since we sample our diffuse texture twice we need to lerp between
        // them to get a smooth transition (shouldn't even be noticable).
        // They start half a cycle apart (0.5) and is later used to calculate
        // the interpolation amount via `2.0 * abs(cycle0 - .5f)`
        self.cycles[0] += 0.25 * delta;
        if self.cycles[0] > 1.0 {
            self.cycles[0] -= 1.0;
        }
        self.cycles[1] += 0.25 * delta;
        if self.cycles[1] > 1.0 {
            self.cycles[1] -= 1.0;
        }

        let (cur_color, _) = self.views[frame.id()].clone();
        self.bundle.data.out = cur_color;

        let mut encoder = pool.acquire_graphics_encoder();
        self.bundle.data.offset0 = self.cycles[0];
        self.bundle.data.offset1 = self.cycles[1];
        let locals = Locals { offsets: self.cycles };
        encoder.update_constant_buffer(&self.bundle.data.locals, &locals);

        encoder.clear(&self.bundle.data.out, [0.3, 0.3, 0.3, 1.0]);
        self.bundle.encode(&mut encoder);
        encoder.synced_flush(queue, &[&sync.rendering], &[], Some(&sync.frame_fence))
            .expect("Could not flush encoder");
    }

    fn on_resize(&mut self, window_targets: gfx_support::WindowTargets<B::Resources>) {
        self.views = window_targets.views;
    }
}

pub fn main() {
    App::launch_simple("Flowmap example");
}
