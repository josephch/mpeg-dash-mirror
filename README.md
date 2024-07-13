# MPEG DASH Mirror

Tool to mirror MPEG dash streams. Downloads manifests and fragments.

## Description

MPEG DASH mirror can be used to download MPEG dash streams. It can be helpful if someone want to host a stream in another webserver. 
Only VOD is supported.
Currently SegmentTemplate based URLs without explicit BaseUrls are supported. Other formats might be added in the future.

## Getting Started

### Getting the source
```
git clone https://github.com/josephch/mpeg-dash-mirror.git
```
### Build and run the source

```
cargo run --release -- --url <url> -o <output directory>
```

## Authors

Contributor name and contact info

Christo Joseph
[twitter](https://x.com/christojoseph)

## License

This project is licensed under the Apache 2.0 - see the LICENSE.md file for details
