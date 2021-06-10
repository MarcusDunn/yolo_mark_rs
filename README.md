# Yolo Mark RS

I've drawn too many boxes in yolo_mark, its usable but has some rough edges:

- Always selects the larger box on hover (cannot delete inner annotations)
- Slow with medium-large sized images
- Left/right arrow increments/decrements both image and tag
- Boxes have no transparency
- No custom keybindings (my implementation still not ideal in this regard but will be improved on)
- You can create 0 sized boxes
- cannot delete 0 sized boxes without clearing all other annotations

So I've resolved these + some quality of life features of my own and this is the result.

![img.png](img.png)

---

# Roadmap

I add features as I personally find certain tasks unergonomic or annoying, as a result this is pretty particular to my
use case. Suggestions and polite feedback are welcome if you find something missing.

- [x] Add a limit to the cache
- [x] Multi-digit names shortcuts
- [x] Change settings in GUI
- [X] Scroll to change name
- [ ] Drag boxes
- [ ] Resize Boxes
- [ ] Make trained network predict boxes and allow the user to correct them (VoTT style)
- [ ] Show some annotation meta-data (labels per class and such)
- [ ] Move to async tasks file IO and image resizing.
- [ ] Label images outside local filesystem
- [ ] Compile to WASM and run on web

# Running

I recommend installing the whole [toolchain](https://rustup.rs/) if you do not have a rust compiler already. Currently,
only runs on Nightly (Once [65439](https://github.com/rust-lang/rust/issues/65439) is on stable, this will no longer be
the case)

__Install Nightly:__\
`rustup toolchain install nightly`

__Compile and Run:__\
`cargo run --release <path to images> <path to names file>`

Once I am updating this less frequently I'll make a point of releasing binaries for Windows and Mac.

On Linux there is also some extra libs needed for [egui](https://github.com/emilk/egui) (the graphics library this is
built on) to work; Debian-based distros you can run the following

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev`

For other distros installing these dependencies is left as an exercise to the reader

# Keybindings

- `W` to go up a name
- `S` to go down
- `A` for prev image
- `D` for next.
- `C` clears all tags
- `R` removes the one you are currently hovered over (highlighted in white)

You can also scroll names with mousewheel (or however you poor trackpad people scroll)
as well as type out the index of the name you want to select (the timing threshold of which can be changed in settings)

# Known Issues

- Despite an image being loaded, it will not display until an event occurs forcing an update.
- Does not respect exif data (such as rotation)

