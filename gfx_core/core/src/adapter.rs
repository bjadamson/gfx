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

//! Logical device adapters.
//!
//! Adapters are the main entry point for opening a [Device](../struct.Device).

use {Backend, Gpu, QueueType};

/// Represents a physical or virtual device, which is capable of running the backend.
pub trait Adapter<B: Backend>: Sized {
    /// Create a new logical gpu with the specified queues.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gfx_core::{Adapter, QueueFamily};
    /// # use gfx_core::dummy::DummyAdapter;
    ///
    /// # let adapter: DummyAdapter = return;
    /// let queue_desc = adapter.get_queue_families()
    ///                         .iter()
    ///                         .map(|&(ref family, ty)|
    ///                             (family, ty, family.num_queues()))
    ///                         .collect::<Vec<_>>();
    /// let gpu = adapter.open(&queue_desc);
    /// ```
    fn open(&self, queue_descs: &[(&B::QueueFamily, QueueType, u32)]) -> Gpu<B>;

    /// Create a new gpu with the specified queues.
    ///
    /// Takes an closure and creates the number of queues for each queue type
    /// as returned by the closure. Queues returning a number of 0 will be filtered out.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gfx_core::{Adapter, QueueType, Surface};
    /// # use gfx_core::dummy::{DummyAdapter, DummySurface};
    ///
    /// # let adapter: DummyAdapter = return;
    /// # let surface: DummySurface = return;
    /// // Open a gpu with a graphics queue, which can be used for presentation.
    /// // GeneralQueues will be downcasted to GraphicsQueues.
    /// let gpu = adapter.open_with(|family, ty| {
    ///     ((ty.supports_graphics() && surface.supports_queue(&family)) as u32, QueueType::Graphics)
    /// });
    ///
    /// ```
    fn open_with<F>(&self, mut f: F) -> Gpu<B>
        where F: FnMut(&B::QueueFamily, QueueType) -> (u32, QueueType)
    {
        let queue_desc = self.get_queue_families()
            .iter()
            .filter_map(|&(ref family, ty)| {
                let (num_queues, ty) = f(family, ty);
                if num_queues > 0 {
                    Some((family, ty, num_queues))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.open(&queue_desc)
    }

    /// Get the `AdapterInfo` for this adapter.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gfx_core::Adapter;
    ///
    /// # let adapter: gfx_core::dummy::DummyAdapter = return;
    /// println!("Adapter info: {:?}", adapter.get_info());
    /// ```
    fn get_info(&self) -> &AdapterInfo;

    /// Return the supported queue families for this adapter.
    ///
    /// * `QueueType` will be the one with the most capabilities.
    /// * There can be multiple families with the same queue type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gfx_core::Adapter;
    ///
    /// # let adapter: gfx_core::dummy::DummyAdapter = return;
    /// for (i, &(_, ty)) in adapter.get_queue_families().into_iter().enumerate() {
    ///     println!("Queue family ({:?}) type: {:?}", i, ty);
    /// }
    /// ```
    fn get_queue_families(&self) -> &[(B::QueueFamily, QueueType)];
}

/// Information about a backend adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct AdapterInfo {
    /// Adapter name
    pub name: String,
    /// Vendor PCI id of the adapter
    pub vendor: usize,
    /// PCI id of the adapter
    pub device: usize,
    /// The device is based on a software rasterizer
    pub software_rendering: bool,
}
