## Film Borders in WASM

If you have a modern browser, you can use the live WASM web version [here](https://app.romnn.com/film-borders).

#### Installation
You can use the [web application](https://app.romnn.com/film-borders) that uses WASM or the CLI tool for batch processing.

To install the CLI, run
```bash
cargo install --git https://github.com/romnn/wasm-film-borders.git --bin film-borders
```

#### Usage
```bash
film-borders apply --image ~/Downloads/testscan.jpg --width 2000 --height 1500 --border 10 --rotate 90
```

For a list of options, see
```bash
film-borders apply --help
```
