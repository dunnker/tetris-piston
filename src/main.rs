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
use graphics::{ Transformed };
use piston::input::Button::{ Keyboard };
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
        const LINE_HEIGHT: f64 = 40f64;
        const STATUS_PREVIEW_GRID_HEIGHT: f64 = CELL_SIZE * 6f64;

        // so that we can access inside clojure
        let use_cache = &mut self.cache;
        let use_tetris = &mut self.tetris;

        self.gl.draw(args.viewport(), |c, gl| {
            // clear the viewport
            graphics::clear(BLACK, gl);

            // render the current score and level
            let mut text = graphics::Text::new(22);
            text.color = ORANGE;
            let mut transform = c.trans(STATUS_LEFT_MARGIN, STATUS_TOP_MARGIN);
            text.draw(&format!("Level: {}", use_tetris.get_level()), use_cache, &c.draw_state, 
                transform.transform, gl);
            transform = transform.trans(0f64, LINE_HEIGHT);

            text.draw(&format!("Score: {}", use_tetris.get_score()), use_cache, &c.draw_state, 
                transform.transform, gl);
            transform = transform.trans(0f64, LINE_HEIGHT);

            // render the next shape as a preview of what's coming next
            for point in use_tetris.get_next_shape().iter() {
                let square = graphics::rectangle::square(0f64, 0f64, CELL_SIZE);
                let color = get_shape_color(use_tetris.get_next_shape_index() as i32);
                // render the shape at col 2 and row 2
                let (x, y) = ((2 as i16 + point.x) as f64 * CELL_SIZE, 
                    (2 as i16 + point.y) as f64 * CELL_SIZE);
                graphics::rectangle(color, square, transform.trans(x, y).transform, gl); 
            }
            transform = transform.trans(0f64, STATUS_PREVIEW_GRID_HEIGHT);

            // render GAME OVER text if necessary
            if use_tetris.get_game_over() {
                text.draw(&"GAME OVER", use_cache, &c.draw_state, 
                    transform.transform, gl);
                transform = transform.trans(0f64, LINE_HEIGHT);

                text.draw(&"Press 'N' for a new game", use_cache, &c.draw_state, 
                    transform.transform, gl);
                transform = transform.trans(0f64, LINE_HEIGHT);

                text.draw(&"Use arrow keys to move and rotate", use_cache, &c.draw_state, 
                    transform.transform, gl);
                transform = transform.trans(0f64, LINE_HEIGHT);
                
                text.draw(&"Press spacebar to drop", use_cache, &c.draw_state, 
                    transform.transform, gl);
                // uncomment if drawing additional text in the status area
                //transform = transform.trans(0f64, LINE_HEIGHT);
            }

            // draw a white border around the game board
            let rect_border = graphics::Rectangle::new_border(WHITE, 1.5);
            rect_border.draw([
                LEFT_MARGIN,
                TOP_MARGIN,
                (CELL_SIZE * COL_COUNT as f64) + 1f64,
                (CELL_SIZE * ROW_COUNT as f64) + 1f64,
            ], &c.draw_state, c.transform, gl);

            // render each cell in the game board
            for col in 0..COL_COUNT as i32 {
                for row in 0..ROW_COUNT as i32 {
                    let cell = use_tetris.get_grid_cell(col, row);
                    if cell.cell_type != GridCellType::Void {
                        let (x, y) = (col as f64 * CELL_SIZE, row as f64 * CELL_SIZE);
                        let square = graphics::rectangle::square(0.0f64, 0.0f64, CELL_SIZE);
                        let color = match cell.cell_type {
                            GridCellType::Shape => get_shape_color(cell.shape_index),
                            GridCellType::Fixed => get_shape_color(cell.shape_index),
                            GridCellType::Ghost => DARK_GRAY,
                            _ => {
                                assert!(false);
                                BLACK
                            }
                        };
                        let transform = c.transform.trans(LEFT_MARGIN, TOP_MARGIN).trans(x, y);
                        graphics::rectangle(color, square, transform, gl);                    
                        let tetromino_border = graphics::Rectangle::new_border(DARK_GRAY, 1.0);
                        tetromino_border.draw([0f64, 0f64, CELL_SIZE, CELL_SIZE], &c.draw_state, transform, gl);
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
            // if the elapsed time is now greater than the time allotted between ticks, then invoke tetris.tick()
            if self.elapsed_time > self.tetris.get_tick_time() as f64 {
                self.elapsed_time = 0.0;
                self.tetris.tick();
            }
        }
    }

    fn handle_input(&mut self, i: Input) {
        match i {
            Input::Press(Keyboard(Key::Left)) => { 
                let col: i32 = self.tetris.get_col();
                self.tetris.set_col(col - 1);
            },

            Input::Press(Keyboard(Key::Right)) => { 
                let col: i32 = self.tetris.get_col();
                self.tetris.set_col(col + 1);
            },

            Input::Press(Keyboard(Key::Up)) => { 
                self.tetris.rotate(true);
            },

            Input::Press(Keyboard(Key::Down)) => { 
                let row: i32 = self.tetris.get_row() + 1;
                self.tetris.set_row(row);
            },

            Input::Press(Keyboard(Key::Space)) => { 
                let mut row: i32 = self.tetris.get_row() + 1;
                while self.tetris.set_row(row) {
                    row += 1;
                }
            },

            Input::Press(Keyboard(Key::N)) => { 
                self.tetris.start_game();
            },

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
        opengl,
        WindowSettings::new("Piston Tetris", [1024, 768]).
            exit_on_esc(true)
    );
    
    let font_path = Path::new("FiraMono-Bold.ttf");

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