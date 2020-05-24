use image::{DynamicImage, GenericImageView};
use wgpu::{
    AddressMode, BufferCopyView, BufferUsage, CommandBuffer, CommandEncoder,
    CommandEncoderDescriptor, CompareFunction, Device, Extent3d, FilterMode, Origin3d, Sampler,
    SamplerDescriptor, TextureCopyView, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsage, TextureView,
};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub fn from_bytes(
        device: &Device,
        bytes: &[u8],
    ) -> Result<(Self, CommandBuffer), failure::Error> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, &img)
    }

    pub fn from_image(
        device: &Device,
        img: &DynamicImage,
    ) -> Result<(Self, CommandBuffer), failure::Error> {
        let rgba = img.as_rgba8().unwrap();
        let dimensions = img.dimensions();

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            label: Some("texture"),
        });

        let buffer =
            device.create_buffer_with_data(&rgba.clone().into_raw(), BufferUsage::COPY_SRC);

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });

        encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * dimensions.0,
                rows_per_image: dimensions.1,
            },
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: Origin3d::ZERO,
            },
            size,
        );

        let cmd_buffer = encoder.finish();
        let view = texture.create_default_view();
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: CompareFunction::Always,
        });

        Ok((
            Self {
                texture,
                view,
                sampler,
            },
            cmd_buffer,
        ))
    }
}
