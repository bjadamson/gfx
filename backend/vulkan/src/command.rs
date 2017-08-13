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

use ash::vk;
use ash::version::DeviceV1_0;
use core::{command, pso, shade, state, target, texture as tex};
use core::{IndexType, VertexCount};
use {Backend, RawDevice, Resources};
use std::sync::Arc;

#[derive(Clone)]
pub struct SubmitInfo {
    pub command_buffer: vk::CommandBuffer,
}

pub struct CommandBuffer {
    pub raw: vk::CommandBuffer,
    pub device: Arc<RawDevice>,
}

impl CommandBuffer {
    fn end(&mut self) -> SubmitInfo {
        unsafe {
            self.device.0.end_command_buffer(self.raw).unwrap(); // TODO: error handling
        }

        SubmitInfo { command_buffer: self.raw }
    }
}

// CommandBuffer trait implementations
impl command::CommandBuffer<Backend> for CommandBuffer {
    unsafe fn end(&mut self) -> SubmitInfo {
        self.end()
    }
}

// TEMPORARY!
impl command::Buffer<Resources> for CommandBuffer {
    fn reset(&mut self) {
        unimplemented!()
    }

    fn bind_pipeline_state(&mut self, _: ()) {
        unimplemented!()
    }

    fn bind_vertex_buffers(&mut self, _: pso::VertexBufferSet<Resources>) {
        unimplemented!()
    }

    fn bind_constant_buffers(&mut self, _: &[pso::ConstantBufferParam<Resources>]) {
        unimplemented!()
    }

    fn bind_global_constant(&mut self, _: shade::Location, _: shade::UniformValue) {
        unimplemented!()
    }

    fn bind_resource_views(&mut self, _: &[pso::ResourceViewParam<Resources>]) {
        unimplemented!()
    }

    fn bind_unordered_views(&mut self, _: &[pso::UnorderedViewParam<Resources>]) {
        unimplemented!()
    }

    fn bind_samplers(&mut self, _: &[pso::SamplerParam<Resources>]) {
        unimplemented!()
    }

    fn bind_pixel_targets(&mut self, _: pso::PixelTargetSet<Resources>) {
        unimplemented!()
    }

    fn bind_index(&mut self, _: (), _: IndexType) {
        unimplemented!()
    }

    fn set_scissor(&mut self, _: target::Rect) {
        unimplemented!()
    }

    fn set_ref_values(&mut self, _: state::RefValues) {
        unimplemented!()
    }

    fn copy_buffer(&mut self,
                   _src: (),
                   _dst: (),
                   _src_offset_bytes: usize,
                   _dst_offset_bytes: usize,
                   _size_bytes: usize) {
        unimplemented!()
    }

    fn copy_buffer_to_texture(&mut self,
                              _src: (),
                              _src_offset_bytes: usize,
                              _dst: (),
                              _kind: tex::Kind,
                              _face: Option<tex::CubeFace>,
                              _img: tex::RawImageInfo) {
        unimplemented!()
    }

    fn copy_texture_to_buffer(&mut self,
                              _src: (),
                              _kind: tex::Kind,
                              _face: Option<tex::CubeFace>,
                              _img: tex::RawImageInfo,
                              _dst: (),
                              _dst_offset_bytes: usize) {
        unimplemented!()
    }

    fn update_buffer(&mut self, _buf: (), _data: &[u8], _offset: usize) {
        unimplemented!()
    }

    fn update_texture(&mut self,
                      _tex: (),
                      _kind: tex::Kind,
                      _face: Option<tex::CubeFace>,
                      _data: &[u8],
                      _image: tex::RawImageInfo) {
        unimplemented!()
    }

    fn generate_mipmap(&mut self, _srv: ()) {
        unimplemented!()
    }

    fn clear_color(&mut self, _target: (), _value: command::ClearColor) {
        unimplemented!()
    }

    fn clear_depth_stencil(&mut self,
                           _target: (),
                           _depth: Option<target::Depth>,
                           _stencil: Option<target::Stencil>) {
        unimplemented!()
    }

    fn call_draw(&mut self,
                 _start: VertexCount,
                 _count: VertexCount,
                 _instances: Option<command::InstanceParams>) {
        unimplemented!();
    }

    fn call_draw_indexed(&mut self,
                         _start: VertexCount,
                         _count: VertexCount,
                         _base: VertexCount,
                         _instances: Option<command::InstanceParams>) {
        unimplemented!()
    }
}

pub struct SubpassCommandBuffer(pub CommandBuffer);

impl command::CommandBuffer<Backend> for SubpassCommandBuffer {
    unsafe fn end(&mut self) -> SubmitInfo {
        self.0.end()
    }
}
