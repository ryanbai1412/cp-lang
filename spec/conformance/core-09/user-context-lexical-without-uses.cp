// expect: ok, output "1 10"
// rule: 09.1.3
// A top-level fn without `uses` reads the context it was written in, like `plain` with `io`.
context opt:
    k: Int = 1
fn lexical() -> Int = opt.k
fn follows() -> Int uses opt = opt.k
with opt(k: 10):
    print(lexical(), follows())
