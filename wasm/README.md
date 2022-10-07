## Film Borders in WASM

If you have a modern browser, you can use the live WASM web version [here](https://film-borders.romnn.com).

#### Installation

You can use the [web application](https://film-borders.romnn.com) that uses WASM or the CLI tool for batch processing.

To install the CLI, run

```bash
cargo install filmborders --bin film-borders
```

For local testing, you can also install the current version locally:

```bash
cargo install --bin film-borders --path .
```

#### Benchmarking

```bash
sudo apt install linux-tools-common linux-tools-generic linux-`tools-name -r`
cargo install flamegraph
sudo cargo flamegraph -o my_flamegraph.svg -- apply --image ./samples/sample1.jpg --output ./output/sample1.png --border 0 --scale 1.00
```

#### Usage

```bash
film-borders --image ~/Downloads/testscan.jpg --width 2000 --height 1500 --border 10 --rotate 90
```

For a list of options, see

```bash
film-borders --help
```

#### Linting

```bash
cargo clippy --tests --benches --examples --all-features -- -Dclippy::all -Dclippy::pedantic
```

#### TODO

- lint
- add rendering text

- consider: where should into be used, where can be passed by reference

- add single simple benchmark
- get out the benchmark data as json or so
- make a performance tracker via github actions
- add a full suite of integration tests that check against expected output

  - should test: different file formats
  - should test: all border fill modes
  - should test: with single image and multiple images
  - should test: different sets of border configurations

- use impls where possible
- inline everything
- add more benchmarks
- add documentation
- add more tests
- add examples
- update the website
- use generic image backend

#### Done

- error handling for lib and border
- implement arithmetic errors for more types
- make the errors more nuanced, e.g. from what operation they stem from
- add backtraces to all error types
- make debug statements console log when on wasm target
- add builder for image such that path can be set more easily
- crop sides should really only be used for the user right?
- subimage rect could also directly return a mutable sub image using crop?
- consider: where should image rect be used
- make all ops checked
- implement scale border mode (not sophisticated)
- split the types into multiple files
- crop to fit should return rect?
- write a clamp trait
- lint
- remove the apply subcommand
- make border optional
- allow custom border images (cli, lib and web)
- make nice ui components
- custom background color
