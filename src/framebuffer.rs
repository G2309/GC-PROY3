use std::f32::INFINITY;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    pub zbuffer: Vec<f32>,
    pub emissive_buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            zbuffer: vec![f32::INFINITY; width * height],
            emissive_buffer: vec![0; width * height],
            background_color: 0x000000, // Black
            current_color: 0xFFFFFF,   // White
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color);
        self.zbuffer.fill(INFINITY);
        self.emissive_buffer.fill(0);
    }

    pub fn point(&mut self, x: usize, y: usize, depth: f32, emit: u32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            if self.zbuffer[index] > depth {
                self.buffer[index] = self.current_color;
                self.zbuffer[index] = depth;
                self.emissive_buffer[index] = emit;
            }
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }
    pub fn set_emission(&mut self, emit: u32) {
        self.emissive_buffer.fill(emit);
    }

    // Set emission for a specific point in the framebuffer
    pub fn set_emission_point(&mut self, x: usize, y: usize, emit: u32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.emissive_buffer[index] = emit;
        }
    }

    pub fn line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize) {
        let mut x0 = x0 as isize;
        let mut y0 = y0 as isize;
        let x1 = x1 as isize;
        let y1 = y1 as isize;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        while x0 != x1 || y0 != y1 {
            // Dibujar punto solo si está dentro de los límites del framebuffer
            if x0 >= 0 && x0 < self.width as isize && y0 >= 0 && y0 < self.height as isize {
                self.point(x0 as usize, y0 as usize, 1.0, 100); // Profundidad fija
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

}

// Add trait implementation for saving to BMP file
pub trait RenderableToFile {
    fn render_buffer(&self, filename: &str) -> io::Result<()>;
}

impl RenderableToFile for Framebuffer {
    fn render_buffer(&self, filename: &str) -> io::Result<()> {
        let path = Path::new(filename);
        let mut file = File::create(&path)?;

        // Write BMP header
        let file_size = 14 + 40 + (self.width * self.height * 4) as u32;
        let reserved: u32 = 0;
        let data_offset: u32 = 14 + 40;
        let header_size: u32 = 40;
        let planes: u16 = 1;
        let bits_per_pixel: u16 = 32;
        let compression: u32 = 0;
        let image_size = (self.width * self.height * 4) as u32;

        file.write_all(b"BM")?;
        file.write_all(&file_size.to_le_bytes())?;
        file.write_all(&reserved.to_le_bytes())?;
        file.write_all(&data_offset.to_le_bytes())?;

        file.write_all(&header_size.to_le_bytes())?;
        file.write_all(&(self.width as u32).to_le_bytes())?;
        file.write_all(&(self.height as u32).to_le_bytes())?;
        file.write_all(&planes.to_le_bytes())?;
        file.write_all(&bits_per_pixel.to_le_bytes())?;
        file.write_all(&compression.to_le_bytes())?;
        file.write_all(&image_size.to_le_bytes())?;
        file.write_all(&0u32.to_le_bytes())?; // X pixels per meter
        file.write_all(&0u32.to_le_bytes())?; // Y pixels per meter
        file.write_all(&0u32.to_le_bytes())?; // Total colors
        file.write_all(&0u32.to_le_bytes())?; // Important colors

        // Write pixel data
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let color = self.buffer[index];
                let r = (color >> 16) & 0xFF;
                let g = (color >> 8) & 0xFF;
                let b = color & 0xFF;
                let a = 0xFF; // Alpha is always 255 (opaque)

                file.write_all(&[b as u8, g as u8, r as u8, a as u8])?;
            }
        }

        Ok(())
    }
}

