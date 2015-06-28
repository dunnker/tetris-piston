extern crate rand;
use rand::Rng;

/// A Point represents a portion of a Shape (or tetromino).
/// There are 4 points per shape, and each point represents
/// an x/y coordinate offset from a center position.
#[derive(Copy, Clone)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

/// The width of the game board
pub const COL_COUNT: u8 = 10;
/// The height of the game board
pub const ROW_COUNT: u8 = 22;

/// The number of tetromino's
pub const SHAPE_COUNT: u8 = 7;
/// The number of points in each tetromino
pub const POINT_COUNT: u8 = 4;

/// The number of rows the player must complete before going to a new level
pub const ROWS_PER_LEVEL: u8 = 10;

/// Each tetromino shape is defined by the SHAPES constant.
/// There are 4 points per shape, and 7 shapes in all.
/// So SHAPES is a two-dimensional array to get access to 
/// each point of each shape.
pub const SHAPES: [[Point; POINT_COUNT as usize]; SHAPE_COUNT as usize] = [

        /*
                      0,-1
                 -1,0 0, 0 1,0
        */
        [Point { x: 0, y: 0 }, Point { x: -1, y: 0 }, Point { x: 0, y: -1 }, Point { x: 1, y: 0 }],

        // see also SQUARE_SHAPE_INDEX const below
        /*
            -1,-1 0,-1
            -1, 0 0, 0
        */
        [Point { x: 0, y: 0 }, Point { x: -1, y: 0 }, Point { x: -1, y: -1 }, Point { x: 0, y: -1 } ],

        /*
               -1,-1 0,-1
                     0, 0 1,0
        */
        [Point { x: 0, y: 0 }, Point { x: 0, y: -1 }, Point { x: -1, y: -1 }, Point { x: 1, y: 0 } ],

        /*
                  0,-1 1,-1
             -1,0 0, 0
        */
        [Point { x: 0, y: 0 }, Point { x: 0, y: -1 }, Point { x: 1, y: -1 }, Point { x: -1, y: 0 } ],

        /*
                      1,-1  
             -1,0 0,0 1, 0
        */
        [Point { x: 0, y: 0 }, Point { x: 1, y: 0 }, Point { x: 1, y: -1 }, Point { x: -1, y: 0 } ],

        /*
             -1,-1  
             -1, 0 0,0 1,0
        */
        [Point { x: 0, y: 0 }, Point { x: -1, y: 0 }, Point { x: -1, y: -1 }, Point { x: 1, y: 0 } ],

        /*
            -2,0, -1,0, 0,0, 1,0
        */
        [Point { x: 0, y: 0 }, Point { x: -1, y: 0 }, Point { x: -2, y: 0 }, Point { x: 1, y: 0 } ]
    ];

/// The square shape is special because it doesn't need to be rotated when the player
/// presses the rotate key
pub const SQUARE_SHAPE_INDEX: i32 = 1;

/// The tetris game board consists of a two-dimensional array of GridCell's. Each GridCell struct
/// contains an enum, GridCellType to indicate the type of cell
#[derive(Copy, Clone, PartialEq)]
pub enum GridCellType { 
    /// A GridCellType can be Void if the cell is empty
    Void, 
    /// Fixed means a point of a shape has been dropped into place.
    Fixed, 
    /// Shape means the cell is a point of a shape that is moving, but not yet dropped.
    Shape,
    /// Ghost means the cell is a point of a "ghost shape", used for previewing where a shape
    /// will be dropped.
    Ghost,
}

/// The tetris game board consists of a two-dimensional array of GridCell's.
#[derive(Copy, Clone)]
pub struct GridCell {
    /// The type of cell enum, see GridCellType
    pub cell_type: GridCellType,
    /// If the type is Fixed or Shape, then shape_index indicates which shape.
    pub shape_index: i32,
}

/// Default GridCell's shape_index to -1 instead of 0
impl Default for GridCell {
    #[inline]
    fn default() -> GridCell {
        GridCell { 
            cell_type: GridCellType::Void, 
            shape_index: -1 
        }
    }
}

/// The Tetris struct maintains current state of the game board (see also GridCell).
/// As each tick() method is called, the current shape advances to the next row. If
/// the shape cannot advance, then the shape becomes fixed to the game board and a new
/// shape is determined at random. To render the game board, users can invoke the
/// method, get_grid_cell(col, row) for each cell to determine what color should be
/// painted at that cell, or paint nothing if the cell is void.
pub struct Tetris {
    /// The game board as a two dimensional array of GridCell's
    grid: [[GridCell; ROW_COUNT as usize]; COL_COUNT as usize],
    /// Game over flag
    game_over: bool,
    /// The current shape equal to the corresponding shape in the SHAPES const
    /// unless the shape has been rotated
    shape: [Point; POINT_COUNT as usize],
    /// The next randomly determined shape, see also next_shape_index
    next_shape: [Point; POINT_COUNT as usize],
    /// The column position of the current moving shape
    col: i32,
    /// The row position of the current moving shape
    row: i32,
    /// The row position where the last ghost shape was determined
    ghost_row: i32,
    /// The current shape index into the SHAPES const
    shape_index: i32,
    /// The next random shape index into the SHAPES const
    next_shape_index: i32,
    /// The current level number
    level: u32,
    /// The number of rows completed for the current level
    rows_completed_level: u8,
    /// The current score
    score: u32,
    /// The total number of rows completed
    rows_completed: u32,
    /// Random number generator
    rng: rand::ThreadRng,
}

impl Tetris {
    /// Constructs a new Tetris struct
    pub fn new() -> Tetris {
        Tetris { 
            grid: [[GridCell::default(); ROW_COUNT as usize]; COL_COUNT as usize],
            game_over: true,
            shape_index: 0,
            next_shape_index: 0,
            shape: SHAPES[0],
            next_shape: SHAPES[0],
            col: 0,
            row: 0,
            ghost_row: 0,
            level: 0,
            score: 0,
            rows_completed: 0,
            rows_completed_level: 0,
            rng: rand::thread_rng(),
        }
    }

    /// Returns true when no more shapes can be added to the game board
    pub fn get_game_over(&self) -> bool {
        self.game_over
    }

    /// Gets the GridCell at the specified col and row. See also GridCell.
    pub fn get_grid_cell(&self, col: i32, row: i32) -> GridCell {
        assert!(col >= 0 && col <= COL_COUNT as i32 - 1);
        assert!(row >= 0 && row <= ROW_COUNT as i32 - 1);
        self.grid[col as usize][row as usize]
    }

    /// Returns the column position of the current shape. Note each Point.x value of the shape
    /// can be added to this column value to determine the actual position of the Point.
    pub fn get_col(&self) -> i32 {
        self.col
    }

    /// When the player presses arrow keys to move the shape left and right, invoke set_col()
    /// to move the shape.
    pub fn set_col(&mut self, col: i32) -> bool {
        if !self.game_over {
            let result: bool = col >= 0 && col < COL_COUNT as i32 && 
                self.valid_location(self.shape, col, self.row, true);
            if result {
                let use_row = self.row;
                // move the current shape, and clear its old position before moving
                self.move_shape(col, use_row, true);
                self.col = col;
            }
            result
        } else {
            false
        }
    }

    /// Returns the row position of the current shape. Note each Point.y value of the shape
    /// can be added to this row value to determine the actual position of the Point.
    pub fn get_row(&self) -> i32 {
        self.row
    }

    /// When the player presses the down arrow to drop the shape, invoke set_row() to set the
    /// new row value.
    pub fn set_row(&mut self, row: i32) -> bool {
        if !self.game_over {
            let result: bool = row >= 0 && row < ROW_COUNT as i32 && 
                self.valid_location(self.shape, self.col, row, true);
            if result {
                let use_col = self.col;
                // move the current shape, and clear its old position before moving
                self.move_shape(use_col, row, true);
                self.row = row;
            }
            result
        } else {
            false
        }
    }

    pub fn get_score(&self) -> u32 {
        self.score
    }

    pub fn get_level(&self) -> u32 {
        self.level
    }

    pub fn get_next_shape(&self) -> [Point; POINT_COUNT as usize] {
        self.next_shape
    }

    pub fn get_next_shape_index(&self) -> i32 {
        self.next_shape_index
    }

    /// Use rotate() when the player presses a key to rotate the current shape.
    pub fn rotate(&mut self, clockwise: bool) -> bool {
        if !self.game_over {
            // rotate a copy of the current shape
            let mut shape = self.shape;
            // there is no need to rotate the square shape as it is symmetrical
            if self.shape_index != SQUARE_SHAPE_INDEX {
                self.rotate_shape(clockwise, &mut shape);
            }
            // if this new shape is in a valid position (not checking sides because we can wall kick)...
            let mut result: bool = self.valid_location(shape, self.col, self.row, false);
            if result {
                // perform wall kick if necessary
                let col = self.wall_kick(shape);
                result = col >= 0;
                if result {
                    // ...then remove the current shape from the board
                    self.clear_shape(); // normally move_shape will take care of this, however, the shape itself is changing (not just position)
                    // ...then assign the copy to the current shape
                    self.shape = shape;
                    self.col = col;
                    let use_row = self.row;
                    // now place the current shape back onto the board
                    self.move_shape(col, use_row, false);
                }
            }
            result
        } else {
            false
        }
    }

    /// Starts a new game by clearing the game board, and resetting the level, score etc.
    pub fn start_game(&mut self) {
        if self.game_over {
            self.game_over = false;
            self.level = 0;
            self.score = 0;
            self.rows_completed = 0;
            self.rows_completed_level = 0;
            self.clear_grid();
            // next shape is a random shape
            self.next_shape_index = self.rng.gen_range(0, SHAPE_COUNT as i32);
            self.next_shape = SHAPES[self.next_shape_index as usize];
            // add a new shape on the board
            self.new_shape();
        }
    }

    /// Advances the state of the game board. Invoke tick() at a time interval related to the current level.
    pub fn tick(&mut self) {
        if !self.game_over {
            let new_row = self.row + 1;
            // if we can't move the shape to a new row...
            if !self.set_row(new_row) {
                // ...then fix the shape into place
                self.shape_to_grid();
                // ...then determine if we completed any rows
                let rows = self.complete_rows();
                // calculate new score
                let score_factor: u16 = match rows {
                    1 => 40,
                    2 => 100,
                    3 => 300,
                    4 => 1200,
                    _ => 0,
                };
                self.score += score_factor as u32 * (self.level + 1);
                // determine if we should start a new level
                if self.rows_completed_level > ROWS_PER_LEVEL {
                    self.rows_completed_level = 0;
                    self.level += 1;
                }
                // ...now place a new shape onto the board
                if !self.new_shape() {
                    self.end_game();
                }
            }
        }
    }

    /// Calculates the time granted between calls to tick(). As the level increases, the amount
    /// of time between ticks grows shorter to make the game more difficult at higher levels.
    pub fn get_tick_time(&self) -> f32 {
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

        if self.level < 10 {
            (STARTING_SLOPE * (self.level - 0) as f32) + STARTING_TIME
        } else {
            (ENDING_SLOPE * (self.level - 10) as f32) + ENDING_TIME
        }
    }

    /// Ends the game. However, the current state of the game is preserved (e.g. not clearing the game board)
    /// because rendering code might still display the board
    pub fn end_game(&mut self) {
        self.game_over = true;
    }

    /* Private methods */

    /// Clear the entire game board
    fn clear_grid(&mut self) {
        self.grid = [[GridCell::default(); ROW_COUNT as usize]; COL_COUNT as usize];
    }

    /// Add a new shape on the board.
    fn new_shape(&mut self) -> bool {
        self.row = 0;
        self.col = COL_COUNT as i32 / 2;
        self.shape_index = self.next_shape_index;
        self.next_shape_index = self.rng.gen_range(0, SHAPE_COUNT as i32);
        self.next_shape = SHAPES[self.next_shape_index as usize];
        self.shape = SHAPES[self.shape_index as usize];
        let result: bool = self.valid_location(self.shape, self.col, self.row, true);
        if result {
            let use_col = self.col;
            let use_row = self.row;
            self.move_shape(use_col, use_row, false); // no need to clear because this is first time on the grid
        }
        result
    }

    /// Compute the actual point on the grid based on a shape point and row, col values
    /// The resulting point may be out of bounds
    fn transform_point(&self, col: i32, row: i32, point: Point) -> Point {
        Point { 
            x: col as i16 + point.x, 
            y: row as i16 + point.y
        }
    }

    /// Given a shape and col, row values, determine if the shape is in a valid position,
    /// keeping in mind that some or all of the points can be out of bounds at the top of the grid.
    fn valid_location(&self, shape: [Point; POINT_COUNT as usize], col: i32, row: i32, check_sides: bool) -> bool {
        let mut result: bool = true;
        // test to see if we can successfully place the shape in the new location...
        for point in shape.iter() {
            let grid_point: Point = self.transform_point(col, row, *point);
            // test points against walls and blocks that are already placed...
            if (check_sides && (grid_point.x < 0 || grid_point.x >= COL_COUNT as i16)) ||
                //grid_point.y < 0 || (it's ok for the y position to be outside grid at the top)
                grid_point.y >= ROW_COUNT as i16 ||
                (self.point_in_bounds(col, row, *point) && 
                    // ok to cast to unsigned after checking in bounds...
                    self.grid[grid_point.x as usize][grid_point.y as usize].cell_type == GridCellType::Fixed) {
                result = false;
                break;
            }
        }
        result
    }

    /// Helper method to iterate over all points of a shape and invoke the supplied
    /// closure, returning each point's corresponding grid cell
    fn for_each_cell<F>(&mut self, shape: [Point; POINT_COUNT as usize], col: i32, row: i32, c: F) 
        where F : Fn(&mut GridCell) {
        for point in shape.iter() {
            if self.point_in_bounds(col, row, *point) {
                let grid_point = self.transform_point(col, row, *point);
                let grid_cell = &mut self.grid[grid_point.x as usize][grid_point.y as usize];
                c(grid_cell);
            }
        }
    }

    /// Place a shape onto the game board at the specified col, row.
    /// If clear_before is true, then base on the shape's current position, 
    /// set the grid cells to Void
    fn move_shape(&mut self, col: i32, row: i32, clear_before: bool) {
        if clear_before {
            self.clear_shape();
        }
        let use_shape = self.shape;
        let use_shape_index = self.shape_index;
        self.for_each_cell(use_shape, col, row, |grid_cell| {
            assert!(grid_cell.cell_type == GridCellType::Void ||
                grid_cell.cell_type == GridCellType::Ghost);
            grid_cell.cell_type = GridCellType::Shape;
            grid_cell.shape_index = use_shape_index as i32;
        });
        // determine the row where ghost shape is placed
        self.ghost_row = row;
        loop {
            if self.valid_location(self.shape, col, self.ghost_row + 1, true) {
                self.ghost_row += 1;
            } else {
                break;
            }
        }
        let use_ghost_row = self.ghost_row;
        self.for_each_cell(use_shape, col, use_ghost_row, |grid_cell| {
            if grid_cell.cell_type == GridCellType::Void {
                grid_cell.cell_type = GridCellType::Ghost;
            }
        });
    }

    /// Determine if a given shape point is within the bounds of the grid relative to col, row
    /// See also SHAPES const which defines each point
    /// See also self.valid_location()
    fn point_in_bounds(&self, col: i32, row: i32, point: Point) -> bool {
        let grid_point: Point = self.transform_point(col, row, point);
        grid_point.x >= 0 &&
            grid_point.x < COL_COUNT as i16 &&
            grid_point.y >= 0 &&
            grid_point.y < ROW_COUNT as i16
    }

    /// For each point of the current shape, relative to the current col, row; set
    /// the GridCell.cell_type to Void. This effectively makes the shape disappear.
    fn clear_shape(&mut self) {
        // clear the shape in its current location...
        let use_shape = self.shape;
        let use_col = self.col;
        let use_row = self.row;
        self.for_each_cell(use_shape, use_col, use_row, |grid_cell| {
            assert!(grid_cell.cell_type == GridCellType::Shape);
            grid_cell.cell_type = GridCellType::Void;
            grid_cell.shape_index = -1;
        });
        let use_ghost_row = self.ghost_row;
        self.for_each_cell(use_shape, use_col, use_ghost_row, |grid_cell| {
            if grid_cell.cell_type == GridCellType::Ghost {
                grid_cell.cell_type = GridCellType::Void;
            }
        });
    }

    /// Invoke this method to anchor the current shape onto the board when it can no longer
    /// advance to a new row. This sets the grid cell's type, for each point of the current shape,
    /// (relative to the current col, row), to Fixed
    fn shape_to_grid(&mut self) {
        let use_shape = self.shape;
        let use_col = self.col;
        let use_row = self.row;
        self.for_each_cell(use_shape, use_col, use_row, |grid_cell| {
            assert!(grid_cell.cell_type == GridCellType::Shape);
            grid_cell.cell_type = GridCellType::Fixed;
        });
    }

    /// Determine if any rows have any gaps, and if they do not, then remove those
    /// rows, and cause all rows above to move down. If a group of rows are collapsed
    /// then a bonus score can be computed.
    fn complete_rows(&mut self) -> u8 {
        let mut result = 0;
        let mut row: i32 = ROW_COUNT as i32 - 1;
        while row >= 0 {
            let row_index: usize = row as usize;
            // look for any void spots on this row
            let found_void = (0..COL_COUNT as usize).any(|c| self.grid[c][row_index].cell_type == GridCellType::Void);
            if !found_void {
                result += 1;
                self.rows_completed_level += 1;
                self.rows_completed += 1;

                // make all cells on this row void
                for col in 0..COL_COUNT as usize {
                    self.grid[col][row_index].cell_type = GridCellType::Void;
                    self.grid[col][row_index].shape_index = -1;
                }

                // bring all rows above row down one...
                for col in 0..COL_COUNT as usize {
                    // iterate in reverse starting from row - 1, back to 0...
                    for temp_row in (0..row as usize).rev() {
                        self.grid[col][temp_row + 1].shape_index = self.grid[col][temp_row].shape_index;
                        self.grid[col][temp_row + 1].cell_type = self.grid[col][temp_row].cell_type;
                    }
                }
            } else {
                row -= 1;
            }
        }
        result
    }

    /// Given a shape, rotate each point of the shape to a new quadrant
    fn rotate_shape(&self, clockwise: bool, shape: &mut [Point; POINT_COUNT as usize]) {
        for point in &mut shape.iter_mut() {
            // transform each point to next quadrant...
            if !clockwise {
                let old_x = point.x;
                point.x = point.y;
                point.y = -1 * old_x;
            } else {
                let old_y = point.y;
                point.y = point.x;
                point.x = -1 * old_y;
            }
        }
    }

    /// Calculate a new column if any of the points of the supplied shape are out of bounds to the left or right
    /// The resulting col position will be offset from the current self.col if a valid location is found,
    /// otherwise -1 is returned
    fn wall_kick(&mut self, shape: [Point; POINT_COUNT as usize]) -> i32 {
        // square piece doesn't rotate, so no need to wall kick
        if self.shape_index != SQUARE_SHAPE_INDEX {
            let mut result: i32 = -1;
            // if on left side of the board, then kick to right, e.g. +1, else -1
            let increment = if self.col < COL_COUNT as i32 / 2 {
                1
            } else {
                -1
            };
            for point in shape.iter() {
                let mut kick_col: i32 = self.col;
                // loop until we've shifted kick_col in bounds for this point's x value
                // after loop, kick_col will be in bounds but not necessarily in valid location
                loop {
                    let grid_x = kick_col + point.x as i32;
                    // if not in bounds, then kick left/right
                    if grid_x < 0 || grid_x >= COL_COUNT as i32 {
                        kick_col += increment;
                    } else {
                        break;
                    }
                }
                // ensure kick_col is a valid location
                // e.g. we may have kicked into a place where there are Fixed cells
                if self.valid_location(shape, kick_col, self.row, true) {
                    result = kick_col;
                    break;
                }
            }
            result
        } else {
            self.col
        }
    }
}