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
use piston::input::Button::{ Keyboard /*, Mouse */ };
use piston::input::Input;
use piston::input::keyboard::Key;

use tetris::*;

pub struct App {
    gl: GlGraphics,
    tetris: Tetris,
    time: f64,
}

const CELL_SIZE: f64 = 30.0;
const LEFT_MARGIN: f64 = 20f64;
const TOP_MARGIN: f64 = 30f64;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        //const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const CYAN: [f32; 4] = [0.0, 1.0, 1.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const ORANGE: [f32; 4] = [1.0, 0.5, 0.0, 1.0];
        const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
        const PURPLE: [f32; 4] = [0.5, 0.0, 1.0, 1.0];

        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear(BLACK, gl);
            let rect_border = graphics::Rectangle::new_border([1.0, 1.0, 1.0, 1.0], 1.0);
            rect_border.draw([
                LEFT_MARGIN,
                TOP_MARGIN,
                CELL_SIZE * COL_COUNT as f64,
                CELL_SIZE * ROW_COUNT as f64,
            ], &c.draw_state, c.transform, gl);

            let text = graphics::Text::new(22);
            //text.draw("hello world", GlyphCache, &c.draw_state, c.transform, gl);
        });

        for col in 0..COL_COUNT {
            for row in 0..ROW_COUNT {
                let cell = self.tetris.get_grid_cell(col, row);
                if cell.cell_type != GridCellType::Void {
                    let (x, y) = (col as f64 * CELL_SIZE + LEFT_MARGIN, row as f64 * CELL_SIZE + TOP_MARGIN);
                    let square = graphics::rectangle::square(x, y, CELL_SIZE);

                    let color = match cell.shape_index {
                        0 => PURPLE,
                        1 => YELLOW,
                        2 => RED,
                        3 => GREEN,
                        4 => ORANGE,
                        5 => BLUE,
                        6 => CYAN,
                        _ => {
                            //assert!(false);
                            BLACK
                        }
                    };

                    self.gl.draw(args.viewport(), |c, gl| {
                        // Draw a box rotating around the middle of the screen.
                        graphics::rectangle(color, square, c.transform, gl);                    
                    });
                }
            }
        }
    }
    
    fn update(&mut self, args: &UpdateArgs) {
        if self.tetris.get_game_over() {
            self.time = 0.0;
        } else {
            self.time += args.dt;
            if self.time > 0.5 {
                self.time = 0.0;
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
                self.tetris.rotate(false);
            },

            Input::Press(Keyboard(Key::Down)) => { 
                let mut row = self.tetris.get_row() + 1;
                while self.tetris.set_row(row) {
                    row += 1;
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
    
    let mut app = App {
        gl: GlGraphics::new(opengl),
        tetris: Tetris::new(),
        time: 0.0,
    };

    app.tetris.start_game();

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