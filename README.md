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

- [ ] Add a limit to the cache (the cache EATS memory if you annotate enough images, you can resize to clear cache for
  now)

- [ ] Multi-digit names shortcuts (type 1 1 quickly to get names #11)

- [ ] Drag boxes

- [ ] Make already trained yolo predict boxes and allow the user to correct them (VoTT style)

- [ ] Use modifier keys in some useful way (control + increment name = increment x 5?)

- [ ] Show some annotation meta-data (labels per class and such)

- [ ] Changing settings in GUI (and more settings such as box thickness and alpha)

- [ ] Label images outside of local filesystem (ssh?)

- [ ] Compile to WASM and run on web (long term)

- [ ] Abstract out traits to allow other export formats and annotation styles

# Running

You'll the rust toolchain to compile. I'll eventually release binaries. Currently, only runs on Nightly

`cargo run --release <path to images> <path to names file>`

On linux there is also some extra libs needed for egui to work. Debian-based distros you will also have to run the
following (for other distros it is left as an exercise to the reader)

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev`
