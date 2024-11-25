mod pov;
mod color;
mod fragment;
mod framebuffer;
mod line;
mod obj;
mod render;
mod shader;
mod vertex;
mod noise;

use crate::pov::POV;
use crate::obj::Obj;
use minifb::{Window, WindowOptions, Key};
use nalgebra_glm::Vec3;
use std::time::Duration;
use std::f32::consts::PI;
use crate::framebuffer::Framebuffer;
use crate::render::{create_model_matrix, create_perspective_matrix, create_view_matrix, create_viewport_matrix, render, Uniforms, gaussian_blur, apply_bloom};
use fastnoise_lite::FastNoiseLite;
use crate::noise::{create_noise, create_cloud_noise};

pub fn start() {
    let window_width = 600;
    let window_height = 600;
    let framebuffer_width = window_width;
    let framebuffer_height = window_height;

    let frame_delay = Duration::from_millis(16);
    let mut framebuffer = Framebuffer::new(window_width, window_height);
    let mut window = Window::new(
        "Planets Orbiting the Sun - Gustavo 22779",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let mut pov = POV::new(
        Vec3::new(15.0, 10.0, 10.0), // Ajusta la posición de la cámara para abarcar todos los planetas
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    framebuffer.set_background_color(20);

    // Datos de los planetas
    let planet_scales = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0]; // Tamaños relativos
    let planet_distances = [3.0, 5.0, 7.0, 9.0, 11.0, 13.0]; // Distancias orbitales
    let planet_speeds = [0.02, 0.015, 0.01, 0.008, 0.006, 0.004]; // Velocidades orbitales

    // Cargar la geometría de la esfera para planetas y sol
    let obj = Obj::load_custom_obj("src/3D/sphere.obj").expect("Failed to load obj");
    let vertex_array = obj.get_vertex_array();

    let model_matrix_sun = create_model_matrix(Vec3::new(0.0, 0.0, 0.0), 1.5, Vec3::new(0.0, 0.0, 0.0)); // El sol
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

    let mut time = 0;

    // RENDER LOOP
    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        handle_input(&window, &mut pov);

        let view_matrix = create_view_matrix(pov.eye, pov.center, pov.up);

        framebuffer.clear();

        let mut uniforms = Uniforms {
            model_matrix: model_matrix_sun,
            view_matrix: &view_matrix,
            projection_matrix: &projection_matrix,
            viewport_matrix: &viewport_matrix,
            time,
            noise: create_noise(1),
            cloud_noise: create_cloud_noise(),
            band_noise: FastNoiseLite::new(),
            current_shader: 7,
        };

        // Renderizar el Sol
        render(&mut framebuffer, &uniforms, &vertex_array, time);

        // Renderizar los planetas
        for (i, (&distance, &scale)) in planet_distances.iter().zip(&planet_scales).enumerate() {
            let angle = time as f32 * planet_speeds[i]; // Calcula el ángulo para cada planeta
            let planet_translation = Vec3::new(
                distance * angle.cos(),
                0.0, // Todos en el mismo plano
                distance * angle.sin(),
            );

            let planet_model_matrix = create_model_matrix(planet_translation, scale, Vec3::new(0.0, 0.0, 0.0));

            uniforms.model_matrix = planet_model_matrix;

            render(&mut framebuffer, &uniforms, &vertex_array, time);
        }

        time += 1;

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}



fn handle_input(window: &Window, pov: &mut POV) {
    const ROTATION_SPEED: f32 = PI / 20.0;
    const ZOOM_SPEED: f32 = 0.75;

    if window.is_key_down(Key::Right) {
        pov.orbit(ROTATION_SPEED, 0.0);
    }
    if window.is_key_down(Key::Left) {
        pov.orbit(-ROTATION_SPEED, 0.0);
    }
    if window.is_key_down(Key::Down) {
        pov.orbit(0.0, -ROTATION_SPEED);
    }
    if window.is_key_down(Key::Up) {
        pov.orbit(0.0, ROTATION_SPEED);
    }

    if window.is_key_down(Key::W) {
        pov.zoom(ZOOM_SPEED);
    }
    if window.is_key_down(Key::S) {
        pov.zoom(-ZOOM_SPEED);
    }
}

fn main() {
    start();
}

