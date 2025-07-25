use image::RgbaImage;
use rayon::prelude::*;
use wgpu::{BindGroup, BindGroupLayout, Device, Extent3d, Queue, TextureFormat, TextureView};

pub struct CubeTextureBuilder;

pub struct CubeTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl CubeTextureBuilder {
    pub fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("cube_texture_bind_group_layout"),
        })
    }

    pub fn create_bind_group(
        device: &Device,
        cube_texture: &CubeTexture,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cube_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&cube_texture.sampler),
                },
            ],
            label: Some("cube_bind_group"),
        })
    }
}

impl CubeTexture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
    pub fn new_shadow_map(
        device: &Device,
        resolution: u32,
        num_lights: u32,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: 6 * num_lights,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::CubeArray),
            array_layer_count: Some(6 * num_lights),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn create_view_from_face(
        &self,
        light_index: u32,
        face_index: u32,
        label: Option<&str>,
    ) -> TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_array_layer: 6 * light_index + face_index,
            array_layer_count: Some(1),
            mip_level_count: Some(1),
            format: Some(wgpu::TextureFormat::Depth32Float),
            ..Default::default()
        })
    }

    pub fn from_files(
        files: &[String],
        device: &Device,
        queue: &Queue,
        label: Option<&str>,
    ) -> Self {
        assert_eq!(files.len(), 6, "Cube maps must contain exactly 6 textures.");
        let rgbas: Vec<RgbaImage> = files
            .par_iter()
            .map(|filename| -> RgbaImage {
                let image = image::open(filename).expect("Failed to load image");
                image.to_rgba8()
            })
            .collect();
        let first_dim = rgbas[0].dimensions();
        for rgba in &rgbas[1..] {
            assert_eq!(
                first_dim,
                rgba.dimensions(),
                "All cubemap faces must be same dimensions"
            );
        }
        let (w, h) = first_dim;
        let size = Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (i, rgba) in rgbas.iter().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                },
                rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * w),
                    rows_per_image: Some(h),
                },
                size,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
