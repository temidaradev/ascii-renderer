# ASCII Renderer

This is a command-line tool written in Rust that converts images into true-color ASCII art directly in your terminal. 

There are plenty of ASCII converters out there, but I noticed a lot of them just grab a single center pixel to figure out what character to draw. That usually makes fine details look distorted and creates weird aliasing patterns. To fix that, this renderer actually downsamples the image mathematically (using Lanczos3 filtering) so the output looks smooth and keeps the original details intact.

## Why use this one?

* **Better resizing:** By using proper filtering from the Rust `image` crate, it perfectly averages and scales pixels down to the terminal grid size. No more moire patterns.
* **True colors:** It prints every single ASCII character in the exact 24-bit RGB color of the source image using `crossterm`.
* **Smart scaling:** It automatically detects your terminal window size and scales the output to fit perfectly. You won't have to deal with awkward line wrapping or scrolling.
* **Aspect ratio correction:** It knows that terminal characters are usually twice as tall as they are wide, and corrects the math so your images don't look squished.
* **Fast rendering:** It queues up all the terminal output and flushes it to the screen in one go, rather than writing character by character.

## Getting Started

You will need to have Rust and Cargo installed on your machine.

1. Clone this repository and navigate into the folder:
   ```bash
   cd ascii-renderer
   ```
2. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

You can run the program and just pass the path to whatever image you want to render.

```bash
cargo run -- path/to/your/image.jpg
```

Or, if you want to use the compiled binary directly:

```bash
./target/release/ascii-renderer path/to/your/image.jpg
```

If you run it without providing a file path, it will look for a default file named `cover.png` in your current folder and try to render that instead.

## How it works under the hood

If you are curious about what the code is actually doing:

1. **Sizing:** The program checks your terminal width and height. It calculates the maximum dimensions for the ASCII grid while keeping the image's original aspect ratio (and accounting for the 1:2 character aspect ratio).
2. **Filtering:** It uses the `resize_exact` function with a `Lanczos3` filter to shrink the original image down to that exact grid size. This ensures the colors are correctly blended.
3. **Brightness:** For every pixel in that new tiny image, it calculates the perceived brightness using the standard relative luminance formula (`0.2126 * R + 0.7152 * G + 0.0722 * B`).
4. **Character mapping:** It takes that brightness value and maps it to a list of ASCII characters that are arranged from darkest to lightest.
5. **Rendering:** Finally, it prints the chosen character to the screen, colorized with the pixel's RGB values.