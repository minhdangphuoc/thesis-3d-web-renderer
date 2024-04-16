use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Cursor};

use anyhow::Ok;
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use crate::model::Material;
use crate::utils::{get_relative_path_from_url, is_gltf_file_url, is_url, Vertex};
use crate::{model, texture};
use std::io::Read;
use url::Url;

fn format_url(path: &str, file_name: &str) -> reqwest::Url {
    let base = Url::parse(path).unwrap();
    base.join(&format!("{path}/{file_name}")).unwrap()
}

pub async fn load_string(path: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url: reqwest::Url;
            if !is_url(path) {
                use crate::utils::log;
                log(&"Url is invalid");
            }
            if is_gltf_file_url(path) {
                url = Url::parse(path).unwrap();
            } else {
                url = format_url(path,"scene.gltf");
            }
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let txt;
            if !is_url(path) {
                let path = std::path::Path::new(env!("OUT_DIR"))
                .join("models/external")
                .join(path)
                .join("scene.gltf");
                txt = std::fs::read_to_string(path)?;
            } else {
                let url : reqwest::Url;
                if is_gltf_file_url(path) {
                    url = Url::parse(path).unwrap();
                } else {
                    url = format_url(path,"scene.gltf");
                }
                txt = reqwest::blocking::get(url)?.text()?;
            }
        }
    }

    Ok(txt)
}

pub async fn load_binary(path: &str, file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url: reqwest::Url;
            if is_url(path) {
                use crate::utils::log;
                log(&"Url is invalid");
            }
            url = format_url(path,file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let data;
            if !is_url(path) {
                let path = std::path::Path::new(env!("OUT_DIR"))
                .join("models/external")
                .join(path)
                .join(file_name);
                data = std::fs::read(path)?;
            } else {
                let url = format_url(path,file_name);
                data = reqwest::blocking::get(url)?.bytes()?.to_vec();
            }
        }
    }

    Ok(data)
}

pub async fn load_texture(
    path: &str,
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    is_albedo_map: bool,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(path, file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_model(
    path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<model::Model> {
    let url = get_relative_path_from_url(path);
    let obj_text = load_string(&path).await?;
    let obj_cursor: Cursor<String> = Cursor::new(obj_text);
    let obj_reader = BufReader::new(obj_cursor);
    let gltf = gltf::Gltf::from_reader(obj_reader)?;

    // Load buffers
    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                if let Some(blob) = gltf.blob.as_deref() {
                    buffer_data.push(blob.into());
                    println!("Found a bin, saving");
                };
            }
            gltf::buffer::Source::Uri(uri) => {
                let bin = load_binary(url, uri).await?;
                buffer_data.push(bin);
            }
        }
    }

    // Load materials
    let mut materials = Vec::new();
    for material in gltf.materials() {
        println!(
            "Loading material: {}",
            material.name().unwrap_or("Undefine").to_string()
        );
        let name = material.name().unwrap_or("Default Material").to_string();
        let pbr: gltf::material::PbrMetallicRoughness<'_> = material.pbr_metallic_roughness();
        let diffuse_factor = pbr.base_color_factor();
        let diffuse_texture: Option<texture::Texture> = match pbr.base_color_texture() {
            Some(texture_info) => {
                match texture_info.texture().source().source() {
                    gltf::image::Source::View { view, .. } => {
                        // Load normal texture from buffer view
                        let texture_source = texture::Texture::from_bytes(
                            device,
                            queue,
                            &buffer_data[view.buffer().index()],
                            url,
                        )
                        .expect("Couldn't load normal texture");
                        Some(texture_source)
                    }
                    gltf::image::Source::Uri { uri, .. } => {
                        // Load normal texture from URI
                        let texture = load_texture(url, uri, device, queue, true)
                            .await
                            .expect("Couldn't load normal texture");
                        Some(texture)
                    }
                }
            }
            None => {
                let default_diffuse_texture =
                    crate::texture::Texture::from_factor(device, queue, &diffuse_factor, url)
                        .expect("Couldn't load normal texture");
                println!("This material doesn't have diffuse(albedo) texture. Creating one");
                Some(default_diffuse_texture)
            }
        };
       

        let diffuse_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Diffuse Factor Buffer"),
            contents: bytemuck::cast_slice(&[diffuse_factor]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut bind_group_entries: Vec<wgpu::BindGroupEntry<'_>> = vec![];
        let mut bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = vec![];

        if let Some(diffuse_texture) = &diffuse_texture {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            });
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            });
            bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            });
            bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }

        // texture_bind_group_layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(
                    &(material.name().unwrap_or("Default Material").to_string()
                        + "text_bind_group_layout"),
                ),
                entries: &bind_group_layout_entries,
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &bind_group_entries,
            label: Some(
                &(material.name().unwrap_or("Default Material").to_string() + "text_bind_group"),
            ),
        });

        materials.push(Material {
            name,
            bind_group,
            texture_bind_group_layout,
            index: materials.len(),
        });
    }

    let mut meshes = Vec::new();

    for mesh in gltf.meshes() {
        let mut mesh_name;
        match mesh.name() {
            Some(name) => {
                println!("Loading mesh: {name} [{}]", mesh.index());
                mesh_name = name;
            }
            None => {
                println!("Loading mesh: Unknown [{}]", mesh.index());
                mesh_name = "Unknown";
            }
        };

        let primitives = mesh.primitives();
        primitives.for_each(|primitive| {
            // dbg!(primitive);

            let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

            let mut vertices = Vec::new();
            if let Some(vertex_attribute) = reader.read_positions() {
                vertex_attribute.for_each(|vertex| {
                    // dbg!(vertex);
                    vertices.push(Vertex {
                        position: vertex,
                        color: Default::default(),
                        tex_coords: Default::default(),
                        normal: Default::default(),
                    })
                });
            }

            if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                let mut tex_coord_index = 0;
                tex_coord_attribute.for_each(|tex_coord| {
                    // dbg!(tex_coord);
                    vertices[tex_coord_index].tex_coords = [tex_coord[0], tex_coord[1]];

                    tex_coord_index += 1;
                });
            }

            if let Some(color_attribute) = reader.read_colors(0) {
                let mut color_index = 0;
                color_attribute.into_rgba_f32().for_each(|color| {
                    vertices[color_index].color = [color[0], color[1], color[2]];
                    color_index += 1;
                });
            }

            let mut indices = Vec::new();
            if let Some(indices_raw) = reader.read_indices() {
                // dbg!(indices_raw);
                indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", path)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", path)),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            // From 1->n+1, Zero if this mesh doesn't use mat
            let mut material = 0;
            match primitive.material().index() {
                Some(_mat) => material = _mat,
                None => {
                    println!("This meshes doesn't use material");
                }
            }

            let mesh_index = primitive.index(); // Test index

            meshes.push(model::Mesh {
                name: mesh_name.to_string(),
                index: mesh_index,
                vertex_buffer,
                index_buffer,
                num_elements: indices.len() as u32,
                material: material as usize,
            });
        });
    }

    for node in gltf.nodes() {
        // Access rotation and translation if available
        let transform = node.transform();
        let mut node_name;
        let mut mesh_index;
        match node.mesh() {
            Some(mesh) => {
                mesh_index = mesh.index();
            }
            None => mesh_index = usize::MAX,
        };
        match node.name() {
            Some(name) => {
                node_name = name;
                println!("Loading node: {name} [{}]", mesh_index);
            }
            None => {
                println!("Loading node: Unknown [{}]", mesh_index);
                node_name = "Unknown"
            }
        };   
    }
    
    Ok(model::Model {
        meshes,
        materials,
    })
}
