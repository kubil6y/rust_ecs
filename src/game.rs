use crate::{
    ecs::{
        components::{RenderComponent, TransformComponent, VelocityComponent},
        registry::Registry,
        systems::{
            movement_system::MovementSystem, render_system::RenderSystem, SystemMaskBuilder,
        },
    },
    logger::Logger,
};
use anyhow::{Error, Result};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::WindowCanvas, Sdl};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, SystemTime},
};

// TODO: map_width and height static mut is unsafe
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
pub const FPS: u128 = 60;
pub const MILLISECS_PER_FRAME: u128 = 1000 / FPS;

pub struct Game {
    is_running: bool,
    prev_frame_time: SystemTime,
    logger: Rc<RefCell<Logger>>,
    registry: Registry,
    sdl_context: Sdl,
    canvas: WindowCanvas,
}

impl Game {
    pub fn new(title: &str) -> Result<Self> {
        let sdl_context = sdl2::init().map_err(Error::msg)?;
        let video_subsystem = sdl_context.video().map_err(Error::msg)?;

        let window = video_subsystem
            .window(title, WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .map_err(Error::msg)?;

        let canvas = window.into_canvas().build().map_err(Error::msg)?;
        let logger = Rc::new(RefCell::new(Logger::default()));
        let registry = Registry::new(Rc::clone(&logger));

        let game = Self {
            is_running: false,
            prev_frame_time: SystemTime::now(),
            registry,
            logger,
            canvas,
            sdl_context,
        };

        Ok(game)
    }

    pub fn run(&mut self) -> Result<()> {
        self.logger.as_ref().borrow_mut().log("Game run!");
        self.setup()?;
        while self.is_running {
            self.process_input()?;
            self.update()?;
            self.render()?;
        }

        Ok(())
    }

    pub fn setup(&mut self) -> Result<()> {
        self.logger
            .as_ref()
            .borrow_mut()
            .log("Game setup is called");
        self.load_level(1)?;
        self.prev_frame_time = SystemTime::now();
        self.is_running = true;
        Ok(())
    }

    pub fn load_level(&mut self, level: i32) -> Result<()> {
        self.registry.register_component::<TransformComponent>()?;
        self.registry.register_component::<RenderComponent>()?;
        self.registry.register_component::<VelocityComponent>()?;

        self.registry.register_system::<MovementSystem>(
            SystemMaskBuilder::new(&self.registry)
                .with::<TransformComponent>()?
                .with::<VelocityComponent>()?
                .build(),
        )?;

        self.registry.register_system::<RenderSystem>(
            SystemMaskBuilder::new(&self.registry)
                .with::<TransformComponent>()?
                .with::<RenderComponent>()?
                .build(),
        )?;

        self.logger
            .as_ref()
            .borrow_mut()
            .log(&format!("Game Level {} is loaded", level));

        Ok(())
    }

    pub fn process_input(&mut self) -> Result<()> {
        let mut event_pump = self.sdl_context.event_pump().map_err(Error::msg)?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    self.is_running = false;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::W => println!("pressed W"),
                    Keycode::S => println!("pressed S"),
                    Keycode::A => println!("pressed A"),
                    Keycode::D => println!("pressed D"),
                    Keycode::Escape => {
                        self.is_running = false;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let wait_time = MILLISECS_PER_FRAME.saturating_sub(
            SystemTime::now()
                .duration_since(self.prev_frame_time)?
                .as_millis(),
        );

        if wait_time > 0 && wait_time <= MILLISECS_PER_FRAME {
            std::thread::sleep(Duration::from_millis(wait_time as u64));
        }

        // delta time is in seconds
        let dt = (SystemTime::now()
            .duration_since(self.prev_frame_time)?
            .as_millis()) as f64
            / 1000.0;

        self.prev_frame_time = SystemTime::now();

        // TODO: check this out
        MovementSystem::update(&mut self.registry, dt)?;

        // Process adding/killing entities to the system by their mask
        self.registry.update();

        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        self.canvas.set_draw_color(Color::RGB(30, 30, 30));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(255, 30, 30));

        self
            .canvas
            .fill_rect(Rect::new(10, 10, 20, 20))
            .map_err(Error::msg)?;

        self.canvas.present();

        Ok(())
    }
}
