use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, SystemTime},
};

use anyhow::{Error, Result};
use sdl2::{pixels::Color, rect::Rect, render::WindowCanvas, Sdl};

use crate::logger::Logger;

// TODO: map_width and height static mut is unsafe
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
pub const FPS: u128 = 60;
pub const MILLISECS_PER_FRAME: u128 = 1000 / FPS;

pub struct Game {
    is_running: bool,
    prev_frame_time: SystemTime,
    level: i32,
    logger: Logger,
    canvas: Rc<RefCell<WindowCanvas>>,
    sdl_context: Sdl,
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
        let mut logger = Logger::new();
        logger.log("Game is created");

        Ok(Self {
            is_running: false,
            prev_frame_time: SystemTime::now(),
            level: 0,
            logger,
            canvas: Rc::new(RefCell::new(canvas)),
            sdl_context,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.setup()?;
        while self.is_running {
            self.process_input();
            self.update()?;
            self.render()?;
        }
        Ok(())
    }

    // game level setup
    pub fn setup(&mut self) -> Result<()> {
        self.prev_frame_time = SystemTime::now();
        self.load_level(1);

        self.is_running = true;
        self.logger.log("Game setup is called");

        Ok(())
    }

    pub fn load_level(&mut self, level: i32) {
        self.level = level;

        self.logger.log(&format!("logger level set to {}", level));
    }

    pub fn process_input(&mut self) {
        self.logger.debug("process_input()");
    }

    pub fn update(&mut self) -> Result<()> {
        let wait_time = MILLISECS_PER_FRAME
            - (SystemTime::now()
                .duration_since(self.prev_frame_time)?
                .as_millis());

        if wait_time > 0 && wait_time <= MILLISECS_PER_FRAME {
            std::thread::sleep(Duration::from_millis(wait_time as u64));
        }

        self.prev_frame_time = SystemTime::now();
        self.logger.debug("update()");

        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        self.logger.debug("render()");

        // TODO:
        self.canvas
            .borrow_mut()
            .fill_rect(Rect::new(50, 50, 50, 50))
            .map_err(Error::msg)?;

        self.canvas
            .borrow_mut()
            .set_draw_color(Color::RGB(30, 30, 30));

        self.canvas.borrow_mut().clear();

        Ok(())
    }
}
