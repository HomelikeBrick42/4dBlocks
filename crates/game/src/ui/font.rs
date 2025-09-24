use cgmath::ElementWise;

use crate::ui::{Quad, Texture, TextureInfo, Ui};
use std::{collections::HashMap, path::Path};

pub struct Font {
    line_height: usize,
    base: usize,
    scale_width: usize,
    scale_height: usize,
    pages: HashMap<usize, Texture>,
    glyphs: HashMap<u32, Glyph>,
}

#[derive(Debug)]
pub struct Glyph {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    xoffset: isize,
    yoffset: isize,
    xadvance: isize,
    page: usize,
}

impl Font {
    pub fn draw_str(
        &self,
        ui: &mut Ui,
        s: &str,
        position: cgmath::Vector2<f32>,
        scale: f32,
        color: cgmath::Vector4<f32>,
    ) {
        let mut width = 0.0;
        {
            for c in s.chars() {
                let Some(glyph) = self.glyphs.get(&(c as u32)) else {
                    continue;
                };
                width += glyph.xadvance as f32 / self.line_height as f32 * scale;
            }
        }

        let mut position = cgmath::vec2(position.x - width * 0.5, position.y);
        for c in s.chars() {
            let Some(glyph) = self.glyphs.get(&(c as u32)) else {
                continue;
            };
            self.draw_glyph(ui, glyph, position, scale, color);
            position.x += glyph.xadvance as f32 / self.line_height as f32 * scale;
        }
    }

    pub fn draw_char(
        &self,
        ui: &mut Ui,
        c: char,
        position: cgmath::Vector2<f32>,
        scale: f32,
        color: cgmath::Vector4<f32>,
    ) -> bool {
        let Some(glyph) = self.glyphs.get(&(c as u32)) else {
            return false;
        };
        self.draw_glyph(ui, glyph, position, scale, color);
        true
    }

    fn draw_glyph(
        &self,
        ui: &mut Ui,
        glyph: &Glyph,
        position: cgmath::Vector2<f32>,
        scale: f32,
        color: cgmath::Vector4<f32>,
    ) {
        let page = self.pages[&glyph.page].clone();

        let size = cgmath::vec2(glyph.width as f32, -(glyph.height as f32))
            / self.line_height as f32
            * scale;
        ui.push_quad(
            Quad {
                position: position
                    + cgmath::vec2(
                        glyph.xoffset as f32,
                        -glyph.yoffset as f32 + self.base as f32,
                    ) / self.line_height as f32
                        * scale
                    + size * 0.5,
                size,
                color,
            },
            Some(TextureInfo {
                texture: page,
                uv_offset: cgmath::vec2(glyph.x as f32, glyph.y as f32).div_element_wise(
                    cgmath::vec2(self.scale_width as f32, self.scale_height as f32),
                ),
                uv_size: cgmath::vec2(glyph.width as f32, glyph.height as f32).div_element_wise(
                    cgmath::vec2(self.scale_width as f32, self.scale_height as f32),
                ),
            }),
        );
    }

    pub fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font: impl AsRef<Path>,
    ) -> std::io::Result<Self> {
        let font_path = font.as_ref();
        let font = std::fs::read_to_string(font_path)?;

        let (page_count,) = font
            .lines()
            .find(|line| line.starts_with("common "))
            .map(|line| (parse_uint(line, "pages=").unwrap(),))
            .unwrap();

        let mut images = HashMap::with_capacity(page_count);
        for line in font.lines() {
            if !line.starts_with("page ") {
                continue;
            }

            let id = parse_uint(line, "id=").unwrap();
            let file = parse_str(line, "file=").unwrap();
            let path = font_path.join(file);
            images.insert(id, std::fs::read(path)?);
        }

        assert_eq!(page_count, images.len());
        Ok(Self::from_raw(device, queue, &font, &images))
    }

    pub fn from_raw(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font: &str,
        font_images: &HashMap<usize, impl AsRef<[u8]>>,
    ) -> Self {
        let (face, unicode, smooth) = font
            .lines()
            .find(|line| line.starts_with("info "))
            .map(|line| {
                (
                    parse_str(line, "face=").unwrap(),
                    parse_uint(line, "unicode=").unwrap(),
                    parse_uint(line, "smooth=").unwrap(),
                )
            })
            .unwrap();
        assert_ne!(unicode, 0);

        let mut pages = HashMap::with_capacity(font_images.len());
        for (&id, image) in font_images {
            let image =
                image::load_from_memory_with_format(image.as_ref(), image::ImageFormat::Png)
                    .unwrap()
                    .to_rgba32f();

            let texture = Texture::new(
                device,
                &format!("{face} Page {id}"),
                image.width(),
                image.height(),
                wgpu::TextureUsages::COPY_DST,
                if smooth == 0 {
                    wgpu::FilterMode::Nearest
                } else {
                    wgpu::FilterMode::Linear
                },
            );
            let t = texture.texture_view().texture();
            queue.write_texture(
                t.as_image_copy(),
                bytemuck::cast_slice(&image),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * 4 * t.width()),
                    rows_per_image: None,
                },
                t.size(),
            );

            pages.insert(id, texture);
        }
        queue.submit(std::iter::empty());

        let (line_height, base, scale_width, scale_height, page_count) = font
            .lines()
            .find(|line| line.starts_with("common "))
            .map(|line| {
                (
                    parse_uint(line, "lineHeight=").unwrap(),
                    parse_uint(line, "base=").unwrap(),
                    parse_uint(line, "scaleW=").unwrap(),
                    parse_uint(line, "scaleH=").unwrap(),
                    parse_uint(line, "pages=").unwrap(),
                )
            })
            .unwrap();
        assert_eq!(page_count, pages.len());

        let glyphs_count = font
            .lines()
            .find(|line| line.starts_with("chars "))
            .and_then(|line| parse_uint(line, "count="))
            .unwrap_or(0);
        let mut glyphs = HashMap::with_capacity(glyphs_count);

        for line in font.lines() {
            if !line.starts_with("char ") {
                continue;
            }

            let id = parse_uint(line, "id=").unwrap() as u32;
            let glyph = Glyph {
                x: parse_uint(line, "x=").unwrap(),
                y: parse_uint(line, "y=").unwrap(),
                width: parse_uint(line, "width=").unwrap(),
                height: parse_uint(line, "height=").unwrap(),
                xoffset: parse_int(line, "xoffset=").unwrap(),
                yoffset: parse_int(line, "yoffset=").unwrap(),
                xadvance: parse_int(line, "xadvance=").unwrap(),
                page: parse_uint(line, "page=").unwrap(),
            };
            assert!(
                pages.contains_key(&glyph.page),
                "page id={} should exist",
                glyph.page
            );
            glyphs.insert(id, glyph);
        }

        Self {
            line_height,
            base,
            scale_width,
            scale_height,
            pages,
            glyphs,
        }
    }
}

fn parse_int(mut s: &str, pat: &str) -> Option<isize> {
    let position = s.find(pat)? + pat.len();
    s = &s[position..];

    let mut len = 0;
    if s.starts_with('-') {
        len += 1;
    }
    while let Some(c) = s[len..].chars().next()
        && c.is_ascii_digit()
    {
        len += c.len_utf8();
    }
    s[..len].parse().ok()
}

fn parse_uint(mut s: &str, pat: &str) -> Option<usize> {
    let position = s.find(pat)? + pat.len();
    s = &s[position..];

    let mut len = 0;
    while let Some(c) = s[len..].chars().next()
        && c.is_ascii_digit()
    {
        len += c.len_utf8();
    }
    s[..len].parse().ok()
}

fn parse_str<'a>(mut s: &'a str, pat: &str) -> Option<&'a str> {
    let position = s.find(pat)? + pat.len();
    s = &s[position..];

    if !s.starts_with('"') {
        return None;
    }
    s = &s[1..];

    let mut len = 0;
    for c in s.chars() {
        if c == '"' {
            break;
        }
        len += c.len_utf8();
    }
    Some(&s[..len])
}
