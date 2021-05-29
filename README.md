# Yolo Mark RS
I've drawn too many boxes in yolo_mark, its usable but has some rough edges
- always selects the larger box on hover (cannot delete inner annotations)
- slow with meduim szed images
- left arrow incrments both image and tag
- no keybindings (my implemention still not ideal in this regard but will be improved on)

So I've resolved these and this is the result.

---
# Roadmap

[ ] multi-digit names shortcuts 
[ ] drag boxes
[ ] changing settings in GUI (and more settings such as box thickness)
[ ] run from web server
[ ] abstract out traits to allow other export formats and annotation formats by third parties

# Running

needs the rust toolchain to compile. I'll eventually release binaries.

`cargo run --release <path to images> <path to names file>`
