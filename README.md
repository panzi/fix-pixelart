# Fix Pixel Art

Tiny program to automatically resize pixel art to it's native resolution.

Uses a very simple "algorithm" to check if all pixels are actually multi pixel
squares and then scales the image appropriately. Supports animated GIFs. Lossy
formats probably don't work, because they kinda smudge the pixels.

**NOTE:** Since the used image library doesn't support reading the animation
repetition value of animated GIFs this will make all GIFs with more than one
frame into infinitely looping GIFs.

**NOTE:** Since the used image library supports reading animated PNGs (APNG)
and animated WebPs, but doesn't support writing them it will only write the
first frame as a still image if the input is an animation and the output is
PNG or WebP.

## Usage

```plain
Usage: fix-pixelart [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   File to resize
  [OUTPUT]  Where to write the output.
            [default: "{basename}.scaled.{ext}"]

Options:
  -i, --in-place            Overwrite the original file.
                            Ignored if an explicit output is defined.
  -f, --only-analyze-first  Only analyze the first frame of an animation.
                            This can lead to a big speed-up, but will create
                            a 1x1 pixel image if the first frame is a blank
                            screen.
  -h, --help                Print help
  -V, --version             Print version
```