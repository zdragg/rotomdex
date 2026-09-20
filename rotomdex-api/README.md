# rotomdex-api

Derived from [mlemesle/rustemon](https://github.com/mlemesle/rustemon), version 4.6.0. 

## Changes made

- Made `#![no_std]`

- Now requires supplying a `CachedTransport` that implements `fn read(&self, &RelativePath) -> Result<Cow<[u8]>, Error>` for creating a Client. See [rotomdex-data](https://github.com/zdragg/rotomdex-data) for data expected to be found under the relative path.

# Original license

See original license under [LICENSE-MIT](LICENSE-MIT).


