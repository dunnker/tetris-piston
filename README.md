# tetris-piston
A tetris game written in Rust using the Piston library

I know the world doesn't need another Tetris clone, however this is a good learning project for me because I've implemented this game in several other languages (Delphi, C#, C++), so it's mostly a matter of translation. But it's also an opportunity to compare Rust to these other languages.

Assuming you already have Rust installed, to build the game unzip the source code and from a command prompt enter:

cargo build

Then to run the game enter:

cargo run

The game has the following features:

* Level difficulty similar to other tetris games
* Scoring based on number of rows completed with bonuses for completing groups of rows at once
* Can preview the next tetromino to appear on the board
* Ghost tetromino lets you know where the current tetromino will be dropped
* Wall kick feature automatically shifts the current tetromino to the left or right when rotating next to the side walls

# Notes about the code
The code contains just two modules, main.rs and tetris.rs  tetris.rs is meant to be a general library for creating a tetris game as it is not dependent on rendering, timers, keyboard events etc. main.rs contains rendering logic and keyboard events -- all provided by Piston.

When I first dug into the code, I was happy to see that Rust supports the abilitiy to create a const array of struct like so:

```rust
pub const SHAPES: [[Point; POINT_COUNT]; SHAPE_COUNT] = [

        /*
                      0,-1
                 -1,0 0, 0 1,0
        */
        [Point { x: 0, y: 0 }, Point { x: -1, y: 0 }, Point { x: 0, y: -1 }, Point { x: 1, y: 0 }],

//...
```

Some languages make you create the array within a method body, so this was impressive to me. Another nice surprise came later when I needed to assign a portion of the SHAPES const to another array:

```rust
        // determine a random shape
        self.next_shape_index = self.rng.gen_range(0, SHAPE_COUNT);
        // assign the shape array from the SHAPES const
        self.next_shape = SHAPES[self.next_shape_index];
```

The assignment was type compatible! In C++ I had to resort to doing a memcpy, which seemed like a bit of a hack.

Also, using Rust's array iterator came in really handy, as the code has many examples like so:

```rust
        for point in self.shape.iter() {
            if self.point_in_bounds(col, row, *point) {
                //..
            }
        }
```

However, on the negative side, I had to pay special attention to integer overflow exceptions which would halt the game abruptly. In other languages I don't remember this coming up. The root of the issue in Rust, is Rust's "usize" datatype, which is an unsigned integer used for indexing into an array. The col and row fields used to track the position of the current tetromino are of type usize. This is because they are often used to index into the grid array which represents the game board. In a few cases I may do a calculation to decrement the row/col value, and if I'm not careful could end up with an overflow if the value goes negative, for example:

```rust
        let col = self.tetris.get_col();
        // !! must check, otherwise could get overflow
        if col > 0 {
            self.tetris.set_col(col - 1);
        }
```

Here is another example that was a bit tedious, also because Rust's while loop does not support do-while:

```rust
    fn complete_rows(&mut self) -> u8 {
        let mut result = 0;
        let mut row: usize = ROW_COUNT - 1;
        loop {
            let found_void = ...
            //..
            if !found_void {
              //..
            } else if row > 0 {
                // row decrements but we must be careful not to overflow as row is unsigned
                row -= 1;
            } else {
                break;
            }
        }
        result
    }
```

In the future I could try keeping data types as signed integers, however, I'm afraid that I will be frequently casting to usize in many places where I'm indexing into arrays. But this may be a worthy compromise.
