# tetris-piston
A tetris game written in Rust using the Piston library

### Binaries

* For windows [download](bin/tetris-piston-version-3a529c0.zip?raw=true)
* For Linux [download](bin/tetris-piston-version-6119499.tar.gz?raw=true)

### About this game

I know the world doesn't need another Tetris clone, however this is a good learning project for me because I've implemented this game in several other languages (Delphi, C#, C++), so it's mostly a matter of translation. But it's also an opportunity to compare Rust to these other languages.

The game has the following features:

* Level difficulty similar to other tetris games
* Scoring based on number of rows completed with bonuses for completing groups of rows at once
* Can preview the next tetromino to appear on the board
* Ghost tetromino lets you know where the current tetromino will be dropped
* Wall kick feature automatically shifts the current tetromino to the left or right when rotating next to the side walls

![Screenshot](Screenshot.png?raw=true "Screenshot")

### Building the project

Assuming you already have Rust installed, to build the game unzip the source code and from a command prompt enter:

cargo build

Then to run the game enter:

cargo run

### Note regarding Windows

To build under Windows, be sure to follow the instructions regarding Freetype for Windows in the [Piston Tutorials/getting-started project](https://github.com/PistonDevelopers/Piston-Tutorials/tree/master/getting-started)

### Notes about the code
The code contains just two modules, main.rs and tetris.rs  
tetris.rs is meant to be a general library for creating a tetris game as it is not dependent on rendering, timers, keyboard events etc.
main.rs contains rendering logic and keyboard events -- all provided by Piston.

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

However, on the negative side, I had to pay special attention to integer overflow exceptions which would halt the game abruptly. In other languages I don't remember this coming up. The issue mostly stemmed from Rust's "usize" datatype, which is an unsigned integer used for indexing into an array. Originally, the col and row fields used to track the position of the current tetromino were of type usize. This was because they were used to index into the grid array which represents the game board. In a few cases I had to do a calculation to decrement the row/col value, and if I wasn't careful I would end up with an overflow if the value went negative, for example:

```rust
        let col = self.tetris.get_col();
        // !! must check, otherwise could get overflow
        if col > 0 {
            self.tetris.set_col(col - 1);
        }
```

In the example above, the value passed to set_col() may be negative when set_col takes a usize data type. This seems fair enough, however, later I found that I had to pay close attention any time I was decrementing an unsigned data type. For example, consider this seemingly benign code:

```rust
fn main() {
    let mut x: u8 = 1;
    let y = -1;
    
    x += y;
    
    println!("{}", x);
}
```

Incrementing unsigned x by the signed y value resulting in a value of 0 doesn't immediately raise alarm bells. But the code will produce a panic and halt. I believe the compiled code is actually executed as:

```rust
        x = x + y as u8;
```

The casting of y as an unsigned type when it has a negative value results in the overflow. I assumed the code would execute more like this:

```rust
        x = (x as i8 + y) as u8;
```

Regardless, I ended up removing usize as the datatype for most of my fields and favored i32 which tended to skirt the overflow issues I was getting. The only caveat being that any time I'm using a field to index into any of my arrays, I have to cast to usize -- but that tends to be a safe operation because by that time I've already done the bounds checking necessary that would avoid an overflow.
