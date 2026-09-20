# rotomdex-gif

Derived from [image-rs/image-gif](https://github.com/image-rs/image-gif), version 0.14.2. 

[`src/gif_decoder.rs`](src/gif_decoder.rs) derived from [image-rs/image](https://github.com/image-rs/image), version 0.25.10.

## Changes made

- Made `#![no_std]`
- Now relies on `embedded_io::BufRead` instead of `std::io::BufReader<R: Read>`.
- Stripped everything except for decoding.

# Original license

See original license under [LICENSE-MIT](LICENSE-MIT).

See `image`'s original license under [LICENSE-image-MIT](LICENSE-image-MIT).
