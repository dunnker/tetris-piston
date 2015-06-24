extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

mod tetris;

use piston::window::WindowSettings;
use piston::event::*;
use glutin_window::GlutinWindow as Glutin_Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use graphics::{ /* Graphics, */ Transformed, /* ImageSize */ };
use piston::input::Button::{ Keyboard /*, Mouse */ };
use piston::input::Input;
use piston::input::keyboard::Key;
use std::path::Path;

use tetris::*;

pub struct App {
    gl: GlGraphics,
    tetris: Tetris,
    elapsed_time: f64,
    cache: GlyphCache<'static>
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
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

impl App {
    fn render(&mut self, args: &RenderArgs) {
        const CELL_SIZE: f64 = 30.0;
        const LEFT_MARGIN: f64 = 20f64;
        const TOP_MARGIN: f64 = 30f64;

        const STATUS_LEFT_MARGIN: f64 = 400f64;
        const STATUS_TOP_MARGIN: f64 = 100f64;
        const LINE_HEIGHT: f64 = 50f64;

        // so that we can access inside clojure
        let temp_cache = &mut self.cache;
        let temp_tetris = &mut self.tetris;

        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear(BLACK, gl);
            let rect_border = graphics::Rectangle::new_border(WHITE, 1.0);
            rect_border.draw([
                LEFT_MARGIN,
                TOP_MARGIN,
                CELL_SIZE * COL_COUNT as f64,
                CELL_SIZE * ROW_COUNT as f64,
            ], &c.draw_state, c.transform, gl);

            let mut text = graphics::Text::new(22);
            text.color = ORANGE;
            text.draw(&format!("Level: {}", temp_tetris.get_level()), temp_cache, &c.draw_state, 
                c.trans(STATUS_LEFT_MARGIN, STATUS_TOP_MARGIN).transform, gl);
            text.draw(&format!("Score: {}", temp_tetris.get_score()), temp_cache, &c.draw_state, 
                c.trans(STATUS_LEFT_MARGIN, STATUS_TOP_MARGIN + LINE_HEIGHT).transform, gl);

            for point in temp_tetris.get_next_shape().iter() {
                let (x, y) = ((2 as i16 + point.x) as f64 * CELL_SIZE + STATUS_LEFT_MARGIN, 
                    (2 as i16 + point.y) as f64 * CELL_SIZE + STATUS_TOP_MARGIN + (LINE_HEIGHT * 2f64));
                let square = graphics::rectangle::square(x, y, CELL_SIZE);
                let color = get_shape_color(temp_tetris.get_next_shape_index() as i32);
                graphics::rectangle(color, square, c.transform, gl);                    
            }

            if temp_tetris.get_game_over() {
                text.draw(&"GAME OVER", temp_cache, &c.draw_state, 
                    c.trans(STATUS_LEFT_MARGIN, STATUS_TOP_MARGIN + (LINE_HEIGHT * 6f64)).transform, gl);
                text.draw(&"Press 'N' for a new game", temp_cache, &c.draw_state, 
                    c.trans(STATUS_LEFT_MARGIN, STATUS_TOP_MARGIN + (LINE_HEIGHT * 7f64)).transform, gl);
            }

            for col in 0..COL_COUNT {
                for row in 0..ROW_COUNT {
                    let cell = temp_tetris.get_grid_cell(col, row);
                    if cell.cell_type != GridCellType::Void {
                        let (x, y) = (col as f64 * CELL_SIZE + LEFT_MARGIN, row as f64 * CELL_SIZE + TOP_MARGIN);
                        let square = graphics::rectangle::square(x, y, CELL_SIZE);
                        let color = match cell.cell_type {
                            GridCellType::Shape => get_shape_color(cell.shape_index),
                            GridCellType::Fixed => get_shape_color(cell.shape_index),
                            GridCellType::Ghost => DARK_GRAY,
                            _ => {
                                assert!(false);
                                BLACK
                            }
                        };
                        graphics::rectangle(color, square, c.transform, gl);                    
                    }
                }
            }
        });
    }
    
    fn update(&mut self, args: &UpdateArgs) {
        if self.tetris.get_game_over() {
            self.elapsed_time = 0.0;
        } else {
            // Here we increment the time elapsed between update()'s
            self.elapsed_time += args.dt;

            // The time it takes for a shape to advance to a new row will be called tick_time.
            // As the level increases we want tick_time to decrease, so the game gets harder and harder.
            // The rate at which we're decreasing will be our slope.
            // The equation of a line is y - y' = m(x - x')
            // ...where m is the slope, x represents the level and y represents the tick_time.
            // Solving for y we get: y = m(x - x') + y'
            // Now, choosing a starting level (x') and an arbitrary starting time (y')
            // we can calculate the tick_time for any level.
            // However, in the later levels (by level 10), we want the slope to decrease, so that the
            // game doesn't get as hard between levels. So after level 9 pick a new slope and time.
            const STARTING_SLOPE: f32 = -0.08f32;
            const STARTING_TIME: f32 = 1.0f32;
            const ENDING_SLOPE: f32 = -0.012f32;
            const ENDING_TIME: f32 = 0.25f32;
            let tick_time = if self.tetris.get_level() < 10 {
                (STARTING_SLOPE * (self.tetris.get_level() - 0) as f32) + STARTING_TIME
            } else {
                (ENDING_SLOPE * (self.tetris.get_level() - 10) as f32) + ENDING_TIME
            };

            // if the time elapsed is now greater than the time between shapes, then 
            if self.elapsed_time > tick_time as f64 {
                self.elapsed_time = 0.0;
                self.tetris.tick();
            }
        }
    }

    fn handle_input(&mut self, i: Input) {
        match i {
            Input::Press(Keyboard(Key::Left)) => { 
                let col = self.tetris.get_col();
                // !! must check, otherwise could get overflow
                if col > 0 {
                    self.tetris.set_col(col - 1);
                }
            },

            Input::Press(Keyboard(Key::Right)) => { 
                let col = self.tetris.get_col();
                self.tetris.set_col(col + 1);
            },

            Input::Press(Keyboard(Key::Up)) => { 
                self.tetris.rotate(true);
            },

            Input::Press(Keyboard(Key::Down)) => { 
                let mut row = self.tetris.get_row() + 1;
                while self.tetris.set_row(row) {
                    row += 1;
                }
            },

            Input::Press(Keyboard(Key::N)) => { 
                if self.tetris.get_game_over() {
                    self.tetris.start_game();
                }
            },

            /*Input::Move(MouseCursor(x, y)) => {
                
            }*/

            _=> {}
        }
    }
}

fn main() {
    start_app();
}

fn start_app() {
    let opengl = OpenGL::_3_2;

    let window = Glutin_Window::new(

        // some versions of glutin require a parameter here?
        //opengl,

        WindowSettings::new("Piston Tetris", [1024, 768]).
            exit_on_esc(true)
    );
    
    let font_path = Path::new("/usr/share/fonts/truetype/freefont/FreeSans.ttf");

    let mut app = App {
        gl: GlGraphics::new(opengl),
        tetris: Tetris::new(),
        elapsed_time: 0.0,
        cache: GlyphCache::new(font_path).unwrap(),
    };  

    for e in window.events() {
        match e {
            Event::Input(i) => {
                app.handle_input(i.clone());
            }

            Event::Render(_) => {
                if let Some(r) = e.render_args() {
                    app.render(&r);
                }
            }

            Event::Update(_) => {
                if let Some(u) = e.update_args() {
                    app.update(&u);
                }
            }

            _ => {}
        } 
    }
}