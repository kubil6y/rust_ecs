use sdl2::pixels::Color;

pub struct TransformComponent {
    pub position: (f64, f64),
}

pub struct RenderComponent {
    pub width: u32,
    pub height: u32,
    pub color: Color,
}

pub struct VelocityComponent(pub f64, pub f64);
