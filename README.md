# My CSV editor

CSV editor for my Rust and WASM study.
(currently can not be save)

[Demo](https://yskszk63.github.io/my-csv-editor/)

## How do it run.

Examples -> ubuntu:21.04

## Precondtion

- Needs build environment.
- Node.js & npm
- git
- Rust [@see](https://rustup.rs/)
- [wasm-pack](https://github.com/rustwasm/wasm-pack)

```
$ sudo apt install build-essential nodejs npm git
...
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
...
```

Install wasm-pack

Download
```
$ TMPDIR=`mktemp -d`
$ curl -L 'https://github.com/rustwasm/wasm-pack/releases/download/v0.9.1/wasm-pack-v0.9.1-x86_64-unknown-linux-musl.tar.gz' | tar -zxf - -C $TMPDIR
...
$ mv $TMPDIR/wasm-pack-v0.9.1-x86_64-unknown-linux-musl/wasm-pack ~/.local/bin/
$ wasm-pack --version
wasm-pack 0.9.1
```

or `cargo install`
```
$ cargo install wasm-pack
...
```

### Clone repository & Initialize

```
$ git clone https://github.com/yskszk63/my-csv-editor
...
$ cd my-csv-editor
$ npm install
...
```

### Run locally

```
$ npm start
```

Listen at http://localhost:8000/

### Build

```
$ npm run build
...
$ ls dist/{*.css,*.wasm,index.*}
dist/bundle.css  dist/fc37c871845c1cfd91f2.module.wasm  dist/index.html  dist/index.js
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
