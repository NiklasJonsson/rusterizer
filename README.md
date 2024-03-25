# Rust software rasterizer

A learning project for understanding the GPU pipeline better.

Features:

* Vertex & fragment shader (in rust)
* Perspective correct interpolation (e.g. texture coordinates)
* MSAA
* Triangle clipping & reconstruction
* Bilinear texture filtering

Run with `cargo run --release`.

## Resources

* [Trip through the graphics pipeline](https://fgiesen.wordpress.com/2011/07/09/a-trip-through-the-graphics-pipeline-2011-index/)
