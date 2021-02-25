extern crate piston_window;
extern crate graphics;
extern crate rand;

pub mod tetris;

use piston_window::*;

use std::path::Path;
use std::fs::OpenOptions;
use tetris::*;

struct App {
    tetris: Tetris,
    elapsed_time: f64,
    glyphs: piston_window::Glyphs
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
    pub fn render_cell(c: &graphics::Context,
        gl: &mut piston_window::G2d, 
        transform: [[f64; 3]; 2], color: [f32; 4]) {
        let square = graphics::rectangle::square(0f64, 0f64, CELL_SIZE - 1f64);
        let mut rectangle = graphics::Rectangle::new(color);
        rectangle.shape = graphics::rectangle::Shape::Round(4.0, 16);
        rectangle.draw(square, &c.draw_state, transform, gl);
    }

    pub fn render_next_shape(c: &graphics::Context, 
        gl: &mut piston_window::G2d, 
        tetris: &Tetris,
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

    pub fn writeln_text<G: Graphics<Texture=gfx_texture::Texture<gfx_device_gl::Resources>>>(text: &str,
        color: piston_window::types::Color, 
        transform: graphics::context::Context, 
        context: &piston_window::Context,
        cache: &mut piston_window::Glyphs, 
        graphics: &mut G) -> graphics::context::Context {
        let mut result: graphics::context::Context = transform;
        Text::new_color(color, TEXT_FONT_SIZE).
            draw(text, cache, &context.draw_state, result.transform, graphics).unwrap();
        result = result.trans(0f64, LINE_HEIGHT);
        result
    } 
    
    pub fn render_game_over_section(c: &graphics::Context, tetris: &Tetris, 
        cache: &mut piston_window::Glyphs, 
        gl: &mut piston_window::G2d, 
        transform: graphics::context::Context) -> graphics::context::Context {
        let mut result: graphics::context::Context = transform;
        result = Render::writeln_text(&"GAME OVER", ORANGE, result, c, cache, gl);

        result = Render::writeln_text(&"Press 'N' for a new game", ORANGE, result, c, cache, gl);

        result = Render::writeln_text(&"Use arrow keys to move and rotate", ORANGE, result, c, cache, gl);
        
        result = Render::writeln_text(&"Press spacebar to drop", ORANGE, result, c, cache, gl);

        result = Render::writeln_text(&format!("Press 'K' to decrease starting level ({})", tetris.get_starting_level()), 
            ORANGE, result, c, cache, gl);

        result = Render::writeln_text(&"Press 'L' to increase starting level", 
            ORANGE, result, c, cache, gl);
        result
    }

    // renders the game board cells e.g. the current shape, ghost shape, and all prior shapes that are
    // fixed in place
    pub fn render_game_board(c: &graphics::Context, 
        gl: &mut piston_window::G2d, tetris: &Tetris) {
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
    fn render(&mut self, window: &mut PistonWindow, event: &impl piston_window::GenericEvent) {
        // so that we can access inside closure
        let use_cache = &mut self.glyphs;
        let use_tetris = &self.tetris;

        window.draw_2d(event, |c, g, device| {
            // clear the viewport
            clear(BLACK, g);

            // render the current score and level
            let mut transform: graphics::context::Context = c.trans(STATUS_LEFT_MARGIN, STATUS_TOP_MARGIN);
            transform = Render::writeln_text(&format!("Level: {}", use_tetris.get_level()), 
                ORANGE, transform, &c, use_cache, g);

            transform = Render::writeln_text(&format!("Score: {}", use_tetris.get_score()), ORANGE, transform, &c, use_cache, g);

            transform = Render::render_next_shape(&c, g, use_tetris, transform);

            // render GAME OVER text if necessary
            if use_tetris.get_game_over() {
                /*transform =*/ Render::render_game_over_section(&c, use_tetris, use_cache, g, transform);
            }

            // draw a white border around the game board
            let rect_border = graphics::Rectangle::new_border(LIGHT_GRAY, 1.5);
            rect_border.draw([
                LEFT_MARGIN - 2f64,
                TOP_MARGIN - 2f64,
                (CELL_SIZE * COL_COUNT as f64) + 3f64,
                (CELL_SIZE * ROW_COUNT as f64) + 3f64,
            ], &c.draw_state, c.transform, g);

            Render::render_game_board(&c, g, use_tetris);

            use_cache.factory.encoder.flush(device);
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
    let mut window: PistonWindow = WindowSettings::new("Piston Tetris", [1024, 768]).
        exit_on_esc(true).
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
        tetris: Tetris::new(),
        elapsed_time: 0.0,
        glyphs: window.load_font(font_path).unwrap(),
    };  

    window.set_lazy(false);
    while let Some(e) = window.next() {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            app.handle_key_input(key);
        };

        if let Some(args) = e.render_args() {
            app.render(&mut window, &e);
        };

        e.update(|args| { app.update(&args); });
    }
}