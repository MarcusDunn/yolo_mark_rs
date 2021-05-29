# Yolo Mark RS
I've drawn too many boxes in yolo_mark, its usable but has some rough edges
- always selects the larger box on hover (cannot delete inner annotations)
- slow with meduim-large sized images
- left/right arrow incrments/decements both image and tag
- boxes have no transparency
- no keybindings (my implemention still not ideal in this regard but will be improved on)

So I've resolved these and this is the result.

---
# Roadmap

- [ ] add a limit to the cache (it munches memory if you annotate enough images, you can resize to clear cache for now)

- [ ] multi-digit names shortcuts (type 1 1 quickly to get names #11)

- [ ] drag boxes

- [ ] changing settings in GUI (and more settings such as box thickness and alpha)

- [ ] run from web server

- [ ] abstract out traits to allow other export formats and annotation styles

# Running

needs the rust toolchain to compile. I'll eventually release binaries.

`cargo run --release <path to images> <path to names file>`
