use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use std::f32::consts::PI;
use fastnoise_lite::FastNoiseLite;
use crate::vertex::Vertex;
use crate::shader::vertex_shader;
use crate::Framebuffer;
use crate::line::triangle_flat_shade;
use crate::color::Color;

pub struct Uniforms <'a>{
    pub model_matrix: Mat4,
    pub view_matrix: &'a Mat4,
    pub projection_matrix: &'a Mat4,
    pub viewport_matrix: &'a Mat4,
    pub time: u32,
    pub noise: FastNoiseLite,
    pub cloud_noise: FastNoiseLite, 
    pub band_noise: FastNoiseLite, 
    pub current_shader: u8, 
}

pub fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], time: u32) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut fragments = Vec::new();
    for tri in &triangles {
        // Pasa los parámetros adicionales `uniforms` y `time` a la función
        fragments.extend(triangle_flat_shade(&tri[0], &tri[1], &tri[2], uniforms, time));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            framebuffer.set_current_color(fragment.color.to_u32());
            framebuffer.point(x, y, fragment.depth, (fragment.intensity * 255.0) as u32);
        }
    }
}

pub fn render_sun(
    framebuffer: &mut Framebuffer,
    uniforms: &mut Uniforms,
    vertex_array: &Vec<Vertex>,
    sun_position: Vec3,
    sun_size: f32,
    time: usize,
) {
    // Set dynamic color and emission for the sun
    let dynamic_emission = 100 + (50.0 * (time as f32 * 0.05).sin()) as u32;
    let dynamic_color = Color::from_float(
        1.0,
        0.9 + 0.1 * (time as f32 * 0.1).cos(),
        0.5 + 0.2 * (time as f32 * 0.1).sin(),
    );

    framebuffer.set_emission(dynamic_emission);
    uniforms.current_shader = 1;

    uniforms.model_matrix = create_model_matrix(sun_position, sun_size, Vec3::new(0.0, 0.0, 0.0));
    render(framebuffer, uniforms, vertex_array, time as u32);
}

fn apply_sun_effects(framebuffer: &mut Framebuffer, width: usize, height: usize) {
    gaussian_blur(&mut framebuffer.emissive_buffer, width, height, 50, 2.0);
    apply_bloom(&mut framebuffer.buffer, &framebuffer.emissive_buffer, width, height);
}

// Matrices transformations

pub fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

pub fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

pub fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 100.0;

    perspective(fov, aspect_ratio, near, far)
}

pub fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

// Blur and Bloom effects

pub fn gaussian_blur(buffer: &mut [u32], width: usize, height: usize, kernel_size: usize, sigma: f32) {
    let gaussian_kernel = create_gaussian_kernel(kernel_size, sigma);
    let kernel_sum: f32 = gaussian_kernel.iter().map(|&x| x as f32).sum();

    // Apply horizontally
    for y in 0..height {
        let mut temp_row = vec![0u32; width];
        for x in 0..width {
            let mut filtered_pixel = 0f32;
            for k in 0..gaussian_kernel.len() {
                let sample_x = x as i32 + k as i32 - (gaussian_kernel.len() / 2) as i32;
                if sample_x >= 0 && sample_x < width as i32 {
                    filtered_pixel += buffer[sample_x as usize + y * width] as f32 * gaussian_kernel[k] as f32;
                }
            }
            temp_row[x] = (filtered_pixel / kernel_sum).round() as u32;
        }
        buffer[y * width..(y + 1) * width].copy_from_slice(&temp_row);
    }

    // Apply vertically
    for x in 0..width {
        let mut temp_col = vec![0u32; height];
        for y in 0..height {
            let mut filtered_pixel = 0f32;
            for k in 0..gaussian_kernel.len() {
                let sample_y = y as i32 + k as i32 - (gaussian_kernel.len() / 2) as i32;
                if sample_y >= 0 && sample_y < height as i32 {
                    filtered_pixel += buffer[x + sample_y as usize * width] as f32 * gaussian_kernel[k] as f32;
                }
            }
            temp_col[y] = (filtered_pixel / kernel_sum).round() as u32;
        }
        for y in 0..height {
            buffer[x + y * width] = temp_col[y];
        }
    }
}

pub fn create_gaussian_kernel(size: usize, sigma: f32) -> Vec<u32> {
    let mut kernel = vec![0u32; size];
    let mean = (size as f32 - 1.0) / 2.0;
    let coefficient = 1.0 / (2.0 * std::f32::consts::PI * sigma * sigma).sqrt();

    for x in 0..size {
        let exp_numerator = -((x as f32 - mean) * (x as f32 - mean)) / (2.0 * sigma * sigma);
        let exp_value = (-exp_numerator).exp();
        kernel[x] = (coefficient * exp_value * 255.0) as u32;
    }

    kernel
}

pub fn apply_bloom(original: &mut [u32], bloom: &[u32], width: usize, height: usize) {
    for i in 0..original.len() {
        let original_color = original[i];
        let bloom_intensity = bloom[i];
        if bloom_intensity > 0 {
            original[i] = blend_bloom(original_color, bloom_intensity);
        }
    }
}

pub fn blend_bloom(base_color: u32, bloom_intensity: u32) -> u32 {
    // Bloom blending
    let bloom_strength = 0.8;
    let max_bloom_effect = 1.2;

    let r = ((base_color >> 16) & 0xFF) as f32;
    let g = ((base_color >> 8) & 0xFF) as f32;
    let b = (base_color & 0xFF) as f32;
    let bloom = bloom_intensity as f32 * bloom_strength;

    // Calculate the new color intensity with clamping
    let new_r = ((r + bloom).min(255.0 * max_bloom_effect)).min(255.0) as u32;
    let new_g = ((g + bloom).min(255.0 * max_bloom_effect)).min(255.0) as u32;
    let new_b = ((b + bloom).min(255.0 * max_bloom_effect)).min(255.0) as u32;

    (new_r << 16) | (new_g << 8) | new_b
}

