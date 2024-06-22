# Fix Pixel Art

Tiny program to automatically resize pixel art to it's native resolution.

Uses a very simple "algorithm" to check if all pixels are actually multi pixel
squares and then scales the image appropriately. Supports animated GIFs. Lossy
formats probably don't work, because they kinda smudge the pixels.

## Usage

```plain
Usage: fix-pixelart [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   File to resize
  [OUTPUT]  Where to write the output. [default: "{input-without-ext}.scaled.{ext}"]

Options:
      --in-place  Overwrite the original file. Ignored if an explicit output is defined
  -h, --help      Print help
  -V, --version   Print version
```
