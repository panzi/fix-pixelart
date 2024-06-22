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

## Examples

| Input | Output |
|:-:|:-:|
| ![Animated GIF of a rotating pixelated coin](https://i.imgur.com/rDBpFfX.gif) | ![Same coin at it's native resolution](https://i.imgur.com/VQdh4aT.gif) |
| ![Pixel art scotch glass being filled](https://i.imgur.com/UMQFFNO.gif) | ![Same scotch glass at it's native resolution](https://i.imgur.com/PgYFKJr.gif) |

Both images &copy; Mathias Panzenböck.

## Usage

```plain
Usage: fix-pixelart [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>
          Image to resize

  [OUTPUT]
          Where to write the output.
          [default: "{basename}.scaled.{ext}"]

Options:
  -i, --in-place
          Overwrite the original file.
          Ignored if an explicit output is defined.

  -a, --only-analyze
          Only analyze the file and print the new size as `{width}x{height}`.
          This can be used if scaling shall be done with a different tool, e.g. ImageMagick:
          
          if size=$(fix-pixelart -a image.gif); then
              convert image.gif -scale "$size" scaled.gif
          fi

  -f, --only-analyze-first
          Only analyze the first frame of an animation.
          This can lead to a big speed-up, but will create a 1x1 pixel image if the first
          frame is a blank screen.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
