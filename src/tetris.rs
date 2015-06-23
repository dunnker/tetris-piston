extern crate rand;
use rand::Rng;

/// A Point represents a portion of a Shape (or tetromino).
/// There are 4 points per shape, and each point represents
/// an x/y coordinate offset from a center position.
#[derive(Copy, Clone)]
pub struct Point {
    x: i16,
    y: i16,
}

pub const COL_COUNT: usize = 10;
pub const ROW_COUNT: usize = 20;

pub const SHAPE_COUNT: usize = 7;
pub const POINT_COUNT: usize = 4;

pub const SHAPES_PER_LEVEL: u8 = 10;

/// Each tetromino shape is defined by the SHAPES constant.
/// There are 4 points per shape, and 7 shapes in all.
/// So SHAPES is a two-dimensional array to get access to 
/// each point of each shape.
pub const SHAPES: [[Point; POINT_COUNT]; SHAPE_COUNT] = [

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

pub const SQUARE_SHAPE_INDEX: usize = 1;

/// The tetris game board consists of a two-dimensional array of GridCell's. Each GridCell struct
/// contains an enum, GridCellType to indicate the type of cell
#[derive(Copy, Clone, PartialEq)]
pub enum GridCellType { 
    /// A GridCellType can be Void if the cell is empty
    Void, 
    /// Fixed means a point of a shape has been dropped into place.
    Fixed, 
    /// Shape means the cell is a point of a shape that is moving, but not yet dropped.
    Shape
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
    grid: [[GridCell; ROW_COUNT]; COL_COUNT],
    game_over: bool,
    shape: [Point; POINT_COUNT],
    next_shape: [Point; POINT_COUNT],
    col: usize,
    row: usize,
    shape_index: usize,
    next_shape_index: usize,
    level: u32,
    shape_level_count: u8,
    score: u32,
    rows_completed: u32,
    rng: rand::ThreadRng,
}

impl Tetris {
    /// Constructs a new Tetris struct
    pub fn new() -> Tetris {
        Tetris { 
            grid: [[GridCell::default(); ROW_COUNT]; COL_COUNT],
            game_over: true,
            shape_index: 0,
            next_shape_index: 0,
            shape: SHAPES[0],
            next_shape: SHAPES[0],
            col: 0,
            row: 0,
            level: 0,
            score: 0,
            rows_completed: 0,
            shape_level_count: 0,
            rng: rand::thread_rng(),
        }
    }

    /// Returns true when no more shapes can be added to the game board
    pub fn get_game_over(&self) -> bool {
        self.game_over
    }

    /// Gets the GridCell at the specified col and row. See also GridCell.
    pub fn get_grid_cell(&self, col: usize, row: usize) -> GridCell {
        assert!(col <= COL_COUNT - 1);
        assert!(row <= ROW_COUNT - 1);
        self.grid[col][row]
    }

    /// Returns the column position of the current shape. Note each Point.x value of the shape
    /// can be added to this column value to determine the actual position of the Point.
    pub fn get_col(&self) -> usize {
        self.col
    }

    /// When the player presses arrow keys to move the shape left and right, invoke set_col()
    /// to move the shape.
    pub fn set_col(&mut self, col: usize) -> bool {
        let result: bool = self.valid_location(self.shape, col, self.row);
        if result {
            let temp_row = self.row;
            // move the current shape, and clear its old position before moving
            self.move_shape(col, temp_row, true);
            self.col = col;
        }
        result
    }

    /// Returns the row position of the current shape. Note each Point.y value of the shape
    /// can be added to this row value to determine the actual position of the Point.
    pub fn get_row(&self) -> usize {
        self.row
    }

    /// When the player presses the down arrow to drop the shape, invoke set_row() to set the
    /// new row value.
    pub fn set_row(&mut self, row: usize) -> bool {
        let result: bool = self.valid_location(self.shape, self.col, row);
        if result {
            let temp_col = self.col;
            // move the current shape, and clear its old position before moving
            self.move_shape(temp_col, row, true);
            self.row = row;
        }
        result
    }

    pub fn get_score(&self) -> u32 {
        self.score
    }

    pub fn get_level(&self) -> u32 {
        self.level
    }

    /// Use rotate() when the player presses a key to rotate the current shape.
    pub fn rotate(&mut self, clockwise: bool) -> bool {
        // rotate a copy of the current shape
        let mut shape = self.shape;
        // there is no need to rotate the square shape as it is symmetrical
        if self.shape_index != SQUARE_SHAPE_INDEX {
            self.rotate_shape(clockwise, &mut shape);
        }
        // if this new shape is in a valid position...
        let result: bool = self.valid_location(shape, self.col, self.row);
        if result {
            // ...then remove the current shape from the board
            self.clear_shape(); // normally move_shape will take care of this, however, the shape itself is changing (not just position)
            // ...then assign the copy to the current shape
            self.shape = shape;
            let temp_col = self.col;
            let temp_row = self.row;
            // now place the current shape back onto the board
            self.move_shape(temp_col, temp_row, false);
        }
        result
    }

    /// Starts a new game by clearing the game board, resetting the level and invoking the first tick().
    pub fn start_game(&mut self) {
        assert!(self.game_over);
        self.game_over = false;
        self.level = 0;
        self.rows_completed = 0;
        self.clear_grid();
        // next shape is a random shape
        self.next_shape_index = self.rng.gen_range(0, SHAPE_COUNT);
        self.next_shape = SHAPES[self.next_shape_index];
        // add a new shape on the board
        self.new_shape();
        self.tick();
    }

    /// Advances the state of the game board. Invoke tick() at a time interval related to the current level.
    pub fn tick(&mut self) {
        assert!(!self.game_over);
        // if we can't move the shape to a new row...
        if !self.new_row() {
            // ...then fix the shape into place
            self.shape_to_grid();
            // ...then determine if we completed any rows
            self.complete_rows();
            // ...now place a new shape onto the board
            if !self.new_shape() {
                self.end_game();
            } else {
                self.shape_level_count += 1;
                if self.shape_level_count > SHAPES_PER_LEVEL {
                    self.new_level();
                }
            }
        }
    }

    /// Ends the game. However, the current state of the game is preserved (e.g. not clearing the game board)
    /// because rendering code might still display the board
    pub fn end_game(&mut self) {
        assert!(!self.game_over);
        self.game_over = true;
    }

    /* Private methods */

    /// Clear the entire game board
    fn clear_grid(&mut self) {
        self.grid = [[GridCell::default(); ROW_COUNT]; COL_COUNT];
    }

    /// Add a new shape on the board.
    fn new_shape(&mut self) -> bool {
        self.row = 0;
        self.col = COL_COUNT / 2;
        self.shape_index = self.next_shape_index;
        self.next_shape_index = self.rng.gen_range(0, SHAPE_COUNT);
        self.next_shape = SHAPES[self.next_shape_index];
        self.shape = SHAPES[self.shape_index];
        let result: bool = self.valid_location(self.shape, self.col, self.row);
        if result {
            //TODO: why required to create local vars here:
            let temp_col = self.col;
            let temp_row = self.row;
            self.move_shape(temp_col, temp_row, false); // no need to clear because this is first time on the grid
        }
        result
    }

    /// Compute the actual point on the grid based on a shape point and row, col values
    fn get_grid_point(&self, col: usize, row: usize, point: Point) -> Point {
        Point { 
            x: col as i16 + point.x as i16, 
            y: row as i16 + point.y as i16
        }
    }

    /// Given a shape and col, row values, determine if the shape is in a valid position,
    /// keeping in mind that some or all of the points can be out of bounds at the top of the grid.
    fn valid_location(&self, shape: [Point; POINT_COUNT], col: usize, row: usize) -> bool {
        let mut result: bool = true;
        // test to see if we can successfully place the shape in the new location...
        for point in shape.iter() {
            let grid_point: Point = self.get_grid_point(col, row, *point);
            // test points that have made it inside the grid,
            // since the shape starts out only partway inside the grid...
            if grid_point.y >= 0 {
                // test points against walls and blocks that are already placed...
                if grid_point.x < 0 ||
                    grid_point.x >= COL_COUNT as i16 ||
                    grid_point.y >= ROW_COUNT as i16 ||
                    // ok to downcast to unsigned because of boolean short circuiting; we already
                    // checked within bounds
                    self.grid[grid_point.x as usize][grid_point.y as usize].cell_type == GridCellType::Fixed {
                    result = false;
                    break;
                }
            } else {
                // if the point is still outside the grid at the top, test to see
                // if the point is inside the grid to the left and right...
                if grid_point.x < 0 ||
                    grid_point.x >= COL_COUNT as i16 {
                    result = false;
                    break;
                }
            }
        }
        result
    }

    /// Place a shape onto the game board at the specified col, row.
    /// If clear_before is true, then base on the shape's current position, 
    /// set the grid cells to Void
    fn move_shape(&mut self, col: usize, row: usize, clear_before: bool) {
        if clear_before {
            self.clear_shape();
        }
        // place the shape in the specified location...
        for point in self.shape.iter() {
            if self.point_in_bounds(col, row, *point) {
                let grid_point: Point = self.get_grid_point(col, row, *point);

                // it's safe to downcast from signed to unsigned since point_in_bounds is true
                let grid_cell = &mut self.grid[grid_point.x as usize][grid_point.y as usize];
                assert!(grid_cell.cell_type == GridCellType::Void);

                grid_cell.cell_type = GridCellType::Shape;
                grid_cell.shape_index = self.shape_index as i32;
            }
        }
    }

    /// Determine if a given shape point is within the bounds of the grid relative to col, row
    /// See also SHAPES const which defines each point
    /// See also self.valid_location()
    fn point_in_bounds(&self, col: usize, row: usize, point: Point) -> bool {
        let grid_point: Point = self.get_grid_point(col, row, point);
        grid_point.x >= 0 &&
            grid_point.x < COL_COUNT as i16 &&
            grid_point.y >= 0 &&
            grid_point.y < ROW_COUNT as i16
    }

    /// For each point of the current shape, relative to the current col, row; set
    /// the GridCell.cell_type to Void. This effectively makes the shape disappear.
    fn clear_shape(&mut self) {
        // clear the shape in its current location...
        for point in self.shape.iter() {
            if self.point_in_bounds(self.col, self.row, *point) {
                let grid_point: Point = self.get_grid_point(self.col, self.row, *point);
                // it's safe to downcast from signed to unsigned since point_in_bounds is true
                let grid_cell = &mut self.grid[grid_point.x as usize][grid_point.y as usize];
                assert!(grid_cell.cell_type == GridCellType::Shape);

                grid_cell.cell_type = GridCellType::Void;
                grid_cell.shape_index = -1;
            }
        }
    }

    /// Attempt to place the current shape one more row below its current row position.
    /// Return true if it is successful.
    fn new_row(&mut self) -> bool {
        let result: bool = self.valid_location(self.shape, self.col, self.row + 1);
        if result {
            //TODO: why required to create local vars here:
            let temp_col = self.col;
            let temp_row = self.row;
            self.move_shape(temp_col, temp_row + 1, true);
            self.row += 1;
        }
        result
    }

    /// Invoke this method to anchor the current shape onto the board when it can no longer
    /// advance to a new row. This sets the grid cell's type, for each point of the current shape,
    /// (relative to the current col, row), to Fixed
    fn shape_to_grid(&mut self) {
        for point in self.shape.iter() {
            if self.point_in_bounds(self.col, self.row, *point) {
                let grid_point: Point = self.get_grid_point(self.col, self.row, *point);
                // it's safe to downcast from signed to unsigned since point_in_bounds is true
                let grid_cell = &mut self.grid[grid_point.x as usize][grid_point.y as usize];

                assert!(grid_cell.cell_type == GridCellType::Shape);

                grid_cell.cell_type = GridCellType::Fixed;
            }
        }
    }

    /// Determine if any rows have any gaps, and if they do not, then remove those
    /// rows, and cause all rows above to move down. If a group of rows are collapsed
    /// then a bonus score can be computed.
    fn complete_rows(&mut self) {
        let mut bonus_rows_completed = 0;
        let mut row: usize = ROW_COUNT - 1;
        loop {
            // look for any void spots on this row
            let mut found_void = false;
            for col in 0..COL_COUNT {
                if self.grid[col][row].cell_type == GridCellType::Void {
                    found_void = true;
                    break;
                }
            }

            if !found_void {
                bonus_rows_completed += 1;
                self.rows_completed += 1;

                // make all cells on this row void
                for col in 0..COL_COUNT {
                    self.grid[col][row].cell_type = GridCellType::Void;
                    self.grid[col][row].shape_index = -1;
                }

                // bring all rows above row down one...
                if row > 0 {
                    for col in 0..COL_COUNT {
                        let mut temp_row: usize = row - 1;
                        loop {
                            self.grid[col][temp_row + 1].shape_index = self.grid[col][temp_row].shape_index;
                            self.grid[col][temp_row + 1].cell_type = self.grid[col][temp_row].cell_type;
                            if temp_row > 0 {
                                // note temp_row is unsigned
                                temp_row -= 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
            } else if row > 0 {
                // row decrements but we must be careful not to overflow as row is unsigned
                row -= 1;
            } else {
                break;
            }
        }

        let score_factor: u16 = match bonus_rows_completed {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => 0,
        };

        self.score += score_factor as u32 * (self.level + 1);
    }

    /// Starts a new level, which affects scoring.
    fn new_level(&mut self) {
        self.shape_level_count = 0;
        self.level += 1;
    }

    /// Given a shape, rotate each point of the shape to a new quadrant
    fn rotate_shape(&self, clockwise: bool, shape: &mut [Point; POINT_COUNT]) {
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
}