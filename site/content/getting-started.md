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

The Rust interpreter and C++ backend are not implemented yet. Use the Python prototype to produce a real C++17 program:

```sh
python3 proto/cp.py build sum.cp
g++ -std=c++17 -O2 sum.cpp -o sum
printf '5\n3 1 4 1 5\n' | ./sum
```

The prototype writes `sum.cpp` beside the source and adds a compiler stamp to `sum.cp`. Work on your own files, not a tracked corpus port. `sum.cpp` is the file you would submit, not the `.cp` source.

## What works today

`cpl check sum.cp` runs the shared Rust frontend and reports the subset of syntax and type errors it understands. A clean check is not proof that a design-only feature is implemented. `cpl lsp` speaks the language server protocol over stdio for editor integrations.

`cpl run sum.cp` and `cpl build sum.cp` currently check the input and then report that their backends are not built. They do not execute or emit C++ yet. The prototype implements more of the language, but is also partial; use the labeled examples in this guide as a starting point.

## The intended contest loop

These are **planned Rust CLI workflows**, not commands to rely on today:

```sh Planned workflow · not implemented in the Rust CLI
cpl run d.cp i1
cpl run d.cp --release
cpl test d.cp
cpl stress d.cp
cpl build d.cp
cpl doc std
```

The target is a fast debug interpreter, stress cases generated from typed input, and one bundled C++17 file with a 64 KB submission-size gate. Local library code should be tested and documented with the same `test` and `doc` commands as the standard library. Until those commands land, follow the [working prototype stress recipe](testing.html#stress-a-real-port-today).
