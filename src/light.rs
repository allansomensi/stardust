//! Dynamic lighting: soft additive glows around shots, explosions, beams and ships.
//!
//! Each frame the game queues light sources with `add` and draws them with `flush`, using
//! a radial texture and an additive blend. A wide, faint pass goes under the ships (it lights
//! the background) and a tighter pass goes on top (it gives shots and blasts a bloom).

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams};
use macroquad::prelude::*;
use std::cell::RefCell;

const VS: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const FS: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
void main() {
    vec4 t = texture2D(Texture, uv);
    gl_FragColor = vec4(color.rgb, color.a * t.a);
}
"#;

struct State {
    mat: Option<Material>,
    tex: Texture2D,
    queue: Vec<(Vec2, f32, Color)>,
    on: bool,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

fn glow_texture() -> Texture2D {
    let n = 64usize;
    let mut bytes = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let dy = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let d = (dx * dx + dy * dy).sqrt().min(1.0);
            let a = (1.0 - d).powf(2.2);
            let i = (y * n + x) * 4;
            bytes[i..i + 3].copy_from_slice(&[255, 255, 255]);
            bytes[i + 3] = (a * 255.0) as u8;
        }
    }
    let t = Texture2D::from_rgba8(n as u16, n as u16, &bytes);
    t.set_filter(FilterMode::Linear);
    t
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let st = s.get_or_insert_with(|| {
            let mat = load_material(
                ShaderSource::Glsl {
                    vertex: VS,
                    fragment: FS,
                },
                MaterialParams {
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::Value(BlendValue::SourceAlpha),
                            BlendFactor::One,
                        )),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .ok();
            State {
                mat,
                tex: glow_texture(),
                queue: Vec::with_capacity(256),
                on: true,
            }
        });
        f(st)
    })
}

/// Turns the lighting on or off (Settings > Graphics).
pub fn set_enabled(on: bool) {
    with(|s| s.on = on);
}

pub fn enabled() -> bool {
    with(|s| s.on && s.mat.is_some())
}

/// Queues a light of radius `r`; the color's alpha is its intensity.
pub fn add(pos: Vec2, r: f32, color: Color) {
    with(|s| {
        if s.on && s.queue.len() < 400 {
            s.queue.push((pos, r, color));
        }
    });
}

/// Draws every queued light, scaled by `radius` and `strength`, and empties the queue
/// unless `keep` is set (so the same lights can be drawn again in a second pass).
pub fn flush(radius: f32, strength: f32, keep: bool) {
    with(|s| {
        let Some(mat) = &s.mat else {
            s.queue.clear();
            return;
        };
        if !s.on || s.queue.is_empty() {
            s.queue.clear();
            return;
        }
        gl_use_material(mat);
        for (p, r, c) in &s.queue {
            let r = r * radius;
            draw_texture_ex(
                &s.tex,
                p.x - r,
                p.y - r,
                Color::new(c.r, c.g, c.b, (c.a * strength).min(1.0)),
                DrawTextureParams {
                    dest_size: Some(vec2(r * 2.0, r * 2.0)),
                    ..Default::default()
                },
            );
        }
        gl_use_default_material();
        if !keep {
            s.queue.clear();
        }
    });
}
