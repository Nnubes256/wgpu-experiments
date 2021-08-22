use std::{
    collections::HashMap,
    marker::PhantomData,
    num::NonZeroU64,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::vertex::{Descriptable, VertexBufferable};
use futures::executor::LocalPool;
use wgpu::{util::DeviceExt, BufferViewMut, CommandEncoder};

// Instead of trying to impl Descriptable for TypedBuffer
// I just define the `descriptor` function for `TypedBuffer` separatedly
// Then constrain T to have `Descriptable`
// And delegate my specific function to `T`'s implementation of `Descriptable::descriptor`
pub trait VertexTypedBuffer<T: VertexBufferable + Descriptable> {
    fn descriptor<'a>(&self) -> wgpu::VertexBufferLayout<'a> {
        T::descriptor()
    }
}

pub struct VertexBuffer<T: VertexBufferable + Descriptable> {
    pub len: u32,
    pub buffer: wgpu::Buffer,
    _t: PhantomData<*mut T>,
}

impl<T> VertexBuffer<T>
where
    T: VertexBufferable + Descriptable,
{
    pub fn from_vertices(device: &wgpu::Device, vertices: &[T], label: Option<&str>) -> Self {
        Self {
            len: vertices.len() as u32,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            _t: PhantomData::default(),
        }
    }
}

impl<T> VertexTypedBuffer<T> for VertexBuffer<T> where T: VertexBufferable + Descriptable {}

pub struct IndexedVertexBuffer<T: VertexBufferable + Descriptable> {
    pub num_indices: u32,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    _t: PhantomData<*mut T>,
}

impl<T> IndexedVertexBuffer<T>
where
    T: VertexBufferable + Descriptable,
{
    pub fn from_vertices_indexes(
        device: &wgpu::Device,
        vertices: &[T],
        indexes: &[u16],
        vertices_label: Option<&str>,
        indexes_label: Option<&str>,
    ) -> Self {
        Self {
            num_indices: indexes.len() as u32,
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: vertices_label,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: indexes_label,
                contents: bytemuck::cast_slice(indexes),
                usage: wgpu::BufferUsages::INDEX,
            }),
            _t: PhantomData::default(),
        }
    }
}

impl<T> VertexTypedBuffer<T> for IndexedVertexBuffer<T> where T: VertexBufferable + Descriptable {}

pub struct InstanceVertexBuffer<T: VertexBufferable + Descriptable> {
    pub len: u32,
    pub buffer: wgpu::Buffer,
    _t: PhantomData<*mut T>,
}

impl<T> InstanceVertexBuffer<T>
where
    T: VertexBufferable + Descriptable,
{
    pub fn descriptor<'a>(&self) -> wgpu::VertexBufferLayout<'a> {
        T::descriptor()
    }

    pub fn from_instances<'a, U>(
        device: &wgpu::Device,
        instances: &'a [U],
        label: Option<&str>,
    ) -> Self
    where
        T: From<&'a U>,
    {
        let t = instances.iter().map(Into::into).collect::<Vec<T>>();
        Self {
            len: instances.len() as u32,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&t),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
            _t: PhantomData::default(),
        }
    }

    pub fn copy_instance<'a, U>(
        &self,
        queue: &wgpu::Queue,
        instance: &'a U,
        index: wgpu::BufferAddress,
    ) where
        T: From<&'a U>,
    {
        let new: T = instance.into();
        let instance_size = std::mem::size_of::<T>() as wgpu::BufferAddress;
        assert!(index < self.len as u64);
        queue.write_buffer(
            &self.buffer,
            index * instance_size,
            bytemuck::bytes_of::<T>(&new),
        )
    }

    pub fn copy_instance_into_view<'a, U>(
        &self,
        buffer: &mut wgpu::BufferViewMut,
        instance: &'a U,
        index: usize,
    ) where
        T: From<&'a U>,
    {
        let new: T = instance.into();
        let instance_size = std::mem::size_of::<T>();
        assert!(index < self.len as usize);
        /* (Mental note: ask about this at some point, seems like an odd omission)

        wgpu has custom address type for buffers, wgpu::BufferAddress. This seems to be a type alias to u64 on all platforms.
        You use that to specify buffer offsets when you write to buffers using most methods.
        However, now that I'm using staging belts, I'm getting BufferViewMut.
        BufferViewMut has nothing, it just derefs to &[u8].
        &[u8] can't be indexed using u64; this means you have to cast to usize, which on 32-bit platforms will panic.
        And also wouldn't even allow you to take advantage of staging buffers that are more than 4 GBs.
        */
        let offset = index * instance_size;
        buffer[offset..(offset + instance_size)].copy_from_slice(bytemuck::bytes_of::<T>(&new))
    }
}

pub trait OldUniform: bytemuck::Pod + bytemuck::Zeroable {
    fn into_buffer(self, device: &wgpu::Device, label: Option<&str>) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::bytes_of(&self),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

pub trait Uniformable: Sized {
    type Uniform: bytemuck::Pod + bytemuck::Zeroable;

    fn into_uniform(self) -> Self::Uniform;

    fn into_buffer(self, device: &wgpu::Device, label: Option<&str>) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::bytes_of(&self.into_uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

pub struct StagingFactory {
    belts: HashMap<String, Mutex<wgpu::util::StagingBelt>>,
    device: Arc<wgpu::Device>,
    local_pool: LocalPool,
}

impl StagingFactory {
    pub fn new(device: &Arc<wgpu::Device>) -> Self {
        Self {
            belts: HashMap::new(),
            device: device.clone(),
            local_pool: LocalPool::new(),
        }
    }

    pub fn create_stager(&mut self, name: String, chunk_size: u64) {
        assert!(
            !self.belts.contains_key(&name),
            "Staging belt \"{}\" was already registered!",
            name
        );
        self.belts
            .insert(name, Mutex::new(wgpu::util::StagingBelt::new(chunk_size)));
    }

    pub fn fetch_stager(&'_ self, name: &str) -> Stager<'_> {
        let belt = self
            .belts
            .get(name)
            .expect("Staging belt \"{}\" not initialized")
            .try_lock()
            .expect("Staging belt \"{}\" already in use");

        assert!(
            self.belts.contains_key(name),
            "Staging belt \"{}\" not initialized",
            name
        );

        Stager {
            device: self.device.clone(),
            belt,
        }
    }

    pub fn submit_all(&mut self) {
        for belt in self.belts.values() {
            let mut belt = belt
                .try_lock()
                .expect("for some reason, this belt is still locked!");
            belt.finish();
        }
    }

    pub fn recall_all(&mut self) {
        for belt in self.belts.values() {
            let mut belt = belt
                .try_lock()
                .expect("for some reason, this belt is still locked!");
            use futures::task::SpawnExt;
            self.local_pool.spawner().spawn(belt.recall()).unwrap();
            self.local_pool.run_until_stalled();
        }
    }
}

pub struct Stager<'factory> {
    device: Arc<wgpu::Device>,
    belt: MutexGuard<'factory, wgpu::util::StagingBelt>,
}

impl<'factory> Stager<'factory> {
    pub fn create_staging_area(
        &mut self,
        encoder: &mut CommandEncoder,
        target: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: NonZeroU64,
    ) -> BufferViewMut {
        self.belt
            .write_buffer(encoder, target, offset, size, &self.device)
    }

    pub fn write_buffer(
        &mut self,
        encoder: &mut CommandEncoder,
        target: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        data: &[u8],
    ) {
        let mut staging_buffer = self.create_staging_area(
            encoder,
            target,
            offset,
            NonZeroU64::new(data.len() as u64).expect("zero sized struct!"),
        );
        staging_buffer.copy_from_slice(data);
    }
}
