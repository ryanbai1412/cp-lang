You need Git, Python 3, a recent stable Rust toolchain, and a C++17 compiler (`g++` for the commands below). The repository is currently private, so cloning requires repository access. No packaged installer or stable release is available yet.

## Install the development CLI

```sh
git clone https://github.com/ryanbai1412/cp-lang.git
cd cp-lang
cargo test
cargo install --path crates/cpl-cli
cpl --version
cpl --help
```

The executable is `cpl`, not `cp`: `cp` already copies files on Unix. Source files use the `.cp` extension. Cargo normally installs the executable in `~/.cargo/bin`; make sure that directory is on your `PATH`.

## Your first program

Save this as `sum.cp`. A program is a function body: top-level statements run in order, with no `main` wrapper or include list.

```example sum
```

`let` binds a value you will not reassign. The type on `n` gives its permitted range. `domain V` makes a fresh type for the positions `1` through `n`; `[V]Int` has one integer per position. `sum` consumes the elements, not their labels.

Input:

```text
5
3 1 4 1 5
```

Output:

```text
14
```

## Run it today

Use the Rust debug interpreter, or emit C++17 and compile a native binary:

```sh
printf '5\n3 1 4 1 5\n' | cpl run sum.cp
cpl build sum.cp
g++ -std=c++17 -O2 sum.cpp -o sum
printf '5\n3 1 4 1 5\n' | ./sum
```

`cpl build` writes `sum.cpp` without modifying the source. The generated C++ includes the original source in a comment header when it fits the 64 KB submission gate; it omits that header with a message if necessary and refuses an oversized C++ file. Submit `sum.cpp`, not the `.cp` source.

For advanced features that currently run only in the Python prototype, copy the source to a scratch directory first, then use `python3 proto/cp.py build scratch/sum.cp`. That prototype adds a compiler stamp to its input and writes `scratch/sum.cpp`; compile the C++ with the same `g++` command.

## What works today

`cpl check sum.cp` runs the shared Rust frontend and reports the subset of syntax and type errors it understands. A clean check is not proof that a design-only feature is implemented. `cpl lsp` speaks the language server protocol over stdio for editor integrations.

`cpl run sum.cp` executes the debug interpreter; `cpl build sum.cp` emits C++17. They share a typed lowering, which reports `E0501` for unsupported constructs. The interpreter checks input ranges, indexing, arithmetic traps, and failed assertions at runtime. The prototype still implements some advanced features missing from Rust; a prototype-tested label is not a claim of Rust support. See the [Rust workspace README](https://github.com/ryanbai1412/cp-lang/blob/main/crates/README.md) for its current coverage.

## The intended contest loop

Basic `cpl run` and `cpl build` work today. These additional workflows and flags are **planned**, not commands to rely on yet:

```sh Planned workflow · not implemented in the Rust CLI
cpl run d.cp i1
cpl run d.cp --release
cpl test d.cp
cpl stress d.cp
cpl doc std
```

The remaining testing goal is stress cases generated from typed input, with local library code tested and documented through the same `test` and `doc` commands as the standard library. Until those commands land, follow the [working prototype stress recipe](testing.html#stress-a-real-port-today).
