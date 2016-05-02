extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

pub mod tetris;

use piston::window::{ AdvancedWindow, WindowSettings };
use glutin_window::GlutinWindow as Window;
use piston::event_loop::*;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use graphics::{ Transformed };
use piston::input::*;

use std::path::Path;
use std::fs::OpenOptions;
use tetris::*;

struct App {
    gl: GlGraphics,
    tetris: Tetris,
    elapsed_time: f64,
    cache: GlyphCache<'static>,
}

//const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const LIGHT_GRAY: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
const DARK_GRAY: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const CYAN: [f32; 4] = [0.0, 1.0, 1.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const ORANGE: [f32; 4] = [1.0, 0.5, 0.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const PURPLE: [f32; 4] = [0.5, 0.0, 1.0, 1.0];

fn get_shape_color(shape_index: i32) -> [f32; 4] {
    match shape_index {
        0 => PURPLE,
        1 => YELLOW,
        2 => RED,
        3 => GREEN,
        4 => ORANGE,
        5 => BLUE,
        6 => CYAN,
        _ => {
            BLACK
        }
    }
}

const TEXT_FONT_SIZE: u32 = 22;
const CELL_SIZE: f64 = 30.0;
const LEFT_MARGIN: f64 = 20f64;
const TOP_MARGIN: f64 = 30f64;

const STATUS_LEFT_MARGIN: f64 = 400f64;
const STATUS_TOP_MARGIN: f64 = 100f64;
const LINE_HEIGHT: f64 = 40f64;
const STATUS_PREVIEW_GRID_HEIGHT: f64 = CELL_SIZE * 6f64;

struct Render;

impl Render {
    pub fn render_cell(c: &graphics::Context, gl: &mut GlGraphics, transform: [[f64; 3]; 2], color: [f32; 4]) {
        let square = graphics::rectangle::square(0f64, 0f64, CELL_SIZE - 1f64);
        let mut rectangle = graphics::Rectangle::new(color);
        rectangle.shape = graphics::rectangle::Shape::Round(4.0, 16);
        rectangle.draw(square, &c.draw_state, transform, gl);
    }

    pub fn render_next_shape(c: &graphics::Context, gl: &mut GlGraphics, tetris: &Tetris,
        transform: graphics::context::Context) -> graphics::context::Context {
        // render the next shape as a preview of what's coming next
        for point in tetris.get_next_shape().iter() {
            let color = get_shape_color(tetris.get_next_shape_index());
            // render the shape at col 2 and row 2
            let (x, y) = ((2 as i16 + point.x) as f64 * CELL_SIZE, 
                (2 as i16 + point.y) as f64 * CELL_SIZE);
            Render::render_cell(&c, gl, transform.trans(x, y).transform, color);
        }
        transform.trans(0f64, STATUS_PREVIEW_GRID_HEIGHT)
    }

    pub fn render_game_over_section(c: &graphics::Context, gl: &mut GlGraphics, cache: &mut GlyphCache<'static>, 
        tetris: &Tetris, transform: graphics::context::Context) -> graphics::context::Context {
        let mut result: graphics::context::Context = transform;
        let mut text = graphics::Text::new(TEXT_FONT_SIZE);
        text.color = ORANGE;
        text.draw(&"GAME OVER", cache, &c.draw_state, result.transform, gl);
        result = result.trans(0f64, LINE_HEIGHT);

        text.draw(&"Press 'N' for a new game", cache, &c.draw_state, result.transform, gl);
        result = result.trans(0f64, LINE_HEIGHT);

        text.draw(&"Use arrow keys to move and rotate", cache, &c.draw_state, 
            result.transform, gl);
        result = result.trans(0f64, LINE_HEIGHT);
        
        text.draw(&"Press spacebar to drop", cache, &c.draw_state, 
            result.transform, gl);
        result = result.trans(0f64, LINE_HEIGHT);

        text.draw(&format!("Press 'K' to decrease starting level ({})", tetris.get_starting_level()), 
            cache, &c.draw_state, result.transform, gl);
        result = result.trans(0f64, LINE_HEIGHT);

        text.draw(&"Press 'L' to increase starting level", 
            cache, &c.draw_state, result.transform, gl);
        result = result.trans(0f64, LINE_HEIGHT);
        result
    }

    // renders the game board cells e.g. the current shape, ghost shape, and all prior shapes that are
    // fixed in place
    pub fn render_game_board(c: &graphics::Context, gl: &mut GlGraphics, tetris: &Tetris) {
        for col in 0..COL_COUNT as i32 {
            for row in 0..ROW_COUNT as i32 {
                let cell = tetris.get_grid_cell(col, row);
                if cell.cell_type != GridCellType::Void {
                    let color = match cell.cell_type {
                        GridCellType::Shape => get_shape_color(cell.shape_index),
                        GridCellType::Fixed => get_shape_color(cell.shape_index),
                        GridCellType::Ghost => DARK_GRAY,
                        _ => {
                            assert!(false);
                            BLACK
                        }
                    };
                    let (x, y) = (col as f64 * CELL_SIZE, row as f64 * CELL_SIZE);
                    let transform = c.transform.trans(LEFT_MARGIN, TOP_MARGIN).trans(x, y);
                    Render::render_cell(&c, gl, transform, color);
                }
            }
        }
    }
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        // so that we can access inside closure
        let use_cache = &mut self.cache;
        let use_tetris = &self.tetris;

        self.gl.draw(args.viewport(), |c, gl| {
            // clear the viewport
            graphics::clear(BLACK, gl);

            // render the current score and level
            let mut text = graphics::Text::new(TEXT_FONT_SIZE);
            text.color = ORANGE;
            let mut transform: graphics::context::Context = c.trans(STATUS_LEFT_MARGIN, STATUS_TOP_MARGIN);
            text.draw(&format!("Level: {}", use_tetris.get_level()), use_cache, &c.draw_state, 
                transform.transform, gl);
            transform = transform.trans(0f64, LINE_HEIGHT);

            text.draw(&format!("Score: {}", use_tetris.get_score()), use_cache, &c.draw_state, 
                transform.transform, gl);
            transform = transform.trans(0f64, LINE_HEIGHT);

            transform = Render::render_next_shape(&c, gl, use_tetris, transform);

            // render GAME OVER text if necessary
            if use_tetris.get_game_over() {
                /*transform =*/ Render::render_game_over_section(&c, gl, use_cache, use_tetris, transform);
            }

            // draw a white border around the game board
            let rect_border = graphics::Rectangle::new_border(LIGHT_GRAY, 1.5);
            rect_border.draw([
                LEFT_MARGIN - 2f64,
                TOP_MARGIN - 2f64,
                (CELL_SIZE * COL_COUNT as f64) + 3f64,
                (CELL_SIZE * ROW_COUNT as f64) + 3f64,
            ], &c.draw_state, c.transform, gl);

            Render::render_game_board(&c, gl, use_tetris);
        });
    }
    
    fn update(&mut self, args: &UpdateArgs) {
        if self.tetris.get_game_over() {
            self.elapsed_time = 0.0;
        } else {
            // Here we increment the time elapsed between update()'s
            self.elapsed_time += args.dt;
            // if the elapsed time is now greater than the time allotted between ticks, then invoke tetris.tick()
            if self.elapsed_time > self.tetris.get_tick_time() as f64 {
                self.elapsed_time = 0.0;
                self.tetris.tick();
            }
        }
    }

    fn handle_key_input(&mut self, key: keyboard::Key) {
        match key {
            Key::Left => { 
                let col: i32 = self.tetris.get_col();
                self.tetris.set_col(col - 1);
            },

            Key::Right => { 
                let col: i32 = self.tetris.get_col();
                self.tetris.set_col(col + 1);
            },

            Key::Up => { 
                self.tetris.rotate(true);
            },

            Key::Down => { 
                let row: i32 = self.tetris.get_row() + 1;
                self.tetris.set_row(row);
            },

            Key::Space => { 
                let mut row: i32 = self.tetris.get_row() + 1;
                while self.tetris.set_row(row) {
                    row += 1;
                }
                // hard drop immediately spawns next shape
                self.tetris.tick();
                self.elapsed_time = 0.0;
            },

            Key::N => { 
                self.tetris.start_game();
            },

            Key::K => { 
                if self.tetris.get_starting_level() > 0 {
                    let new_level: u32 = self.tetris.get_starting_level() - 1;
                    self.tetris.set_starting_level(new_level); 
                }
            },

            Key::L => { 
                if self.tetris.get_starting_level() < 30 {
                    let new_level: u32 = self.tetris.get_starting_level() + 1;
                    self.tetris.set_starting_level(new_level); 
                }
            },

            _ => { }
        }
    }
}

fn main() {
    start_app();
}

fn start_app() {
    let opengl = OpenGL::V2_1;

    let mut window: Window = WindowSettings::new("Piston Tetris", [1024, 768]).
        exit_on_esc(true).
        opengl(opengl).
        build().
        unwrap();
    
    let font_path = match OpenOptions::new().read(true).open("FiraMono-Bold.ttf") {
        Ok(_) => Path::new("FiraMono-Bold.ttf"),
        Err(_) => {
            match OpenOptions::new().read(true).open("src/FiraMono-Bold.ttf") {
                Ok(_) => Path::new("src/FiraMono-Bold.ttf"),
                Err(_) => panic!("Font file is missing, or does not exist in the current path."),
            }
        }
    };

    let mut app = App {
        gl: GlGraphics::new(opengl),
        tetris: Tetris::new(),
        elapsed_time: 0.0,
        cache: GlyphCache::new(font_path).unwrap(),
    };  

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            app.handle_key_input(key);
        };

        if let Some(args) = e.render_args() {
            app.render(&args);
        };

        e.update(|args| { app.update(&args); });
    }
}