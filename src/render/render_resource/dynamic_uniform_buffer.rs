use bevy::render::render_resource::encase::DynamicUniformBuffer as DynamicUniformBufferWrapper;
use bevy::render::{
    render_resource::{encase::private::WriteInto, Buffer, ShaderType},
    renderer::{RenderDevice, RenderQueue},
};
use wgpu::{util::BufferInitDescriptor, BindingResource, BufferBinding, BufferUsages};

pub struct DynamicUniformBuffer<T: ShaderType> {
    values: Vec<T>,
    scratch: DynamicUniformBufferWrapper<Vec<u8>>,
    buffer: Option<Buffer>,
    capacity: usize,
    label: Option<String>,
    changed: bool,
    buffer_usage: BufferUsages,
}

impl<T: ShaderType> Default for DynamicUniformBuffer<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            scratch: DynamicUniformBufferWrapper::new(Vec::new()),
            buffer: None,
            capacity: 0,
            label: None,
            changed: false,
            buffer_usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        }
    }
}

impl<T: ShaderType + WriteInto> DynamicUniformBuffer<T> {
    pub fn new_with_alignment(alignment: u64) -> Self {
        Self {
            values: Vec::new(),
            scratch: DynamicUniformBufferWrapper::new_with_alignment(Vec::new(), alignment),
            buffer: None,
            capacity: 0,
            label: None,
            changed: false,
            buffer_usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        }
    }

    #[inline]
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    #[inline]
    pub fn binding(&self) -> Option<BindingResource> {
        Some(BindingResource::Buffer(BufferBinding {
            buffer: self.buffer()?,
            offset: 0,
            size: Some(T::min_size()),
        }))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Push data into the `DynamicUniformBuffer`'s internal vector (residing on system RAM).
    #[inline]
    pub fn push(&mut self, value: T) -> u32 {
        let offset = self.scratch.write(&value).unwrap() as u32;
        self.values.push(value);
        offset
    }

    pub fn set_label(&mut self, label: Option<&str>) {
        let label = label.map(str::to_string);

        if label != self.label {
            self.changed = true;
        }

        self.label = label;
    }

    pub fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Add more [`BufferUsages`] to the buffer.
    ///
    /// This method only allows addition of flags to the default usage flags.
    ///
    /// The default values for buffer usage are `BufferUsages::COPY_DST` and `BufferUsages::UNIFORM`.
    pub fn add_usages(&mut self, usage: BufferUsages) {
        self.buffer_usage |= usage;
        self.changed = true;
    }

    /// Queues writing of data from system RAM to VRAM using the [`RenderDevice`](crate::renderer::RenderDevice)
    /// and the provided [`RenderQueue`](crate::renderer::RenderQueue).
    ///
    /// If there is no GPU-side buffer allocated to hold the data currently stored, or if a GPU-side buffer previously
    /// allocated does not have enough capacity, a new GPU-side buffer is created.
    #[inline]
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        let size = self.scratch.as_ref().len();

        if self.capacity < size || self.changed {
            self.buffer = Some(device.create_buffer_with_data(&BufferInitDescriptor {
                label: self.label.as_deref(),
                usage: self.buffer_usage,
                contents: self.scratch.as_ref(),
            }));
            self.capacity = size;
            self.changed = false;
        } else if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, self.scratch.as_ref());
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.values.clear();
        self.scratch.as_mut().clear();
        self.scratch.set_offset(0);
    }
}
