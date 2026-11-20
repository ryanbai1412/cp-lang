// Perf Repro #9 (P1 — RSS #1 dmopc21c2p2 37.42x = 131 MB vs 3.5 MB;
//                also RSS #4 ccc21s2 7.25x, #7 ioi09p3 5.67x, #8 noi19p5 5.43x, #11 apio09p1 5.02x):
// `cpl::read_all()` reads the entire stdin stream into a single `std::string` before parsing,
// and `cpl::out` buffers the entire stdout in RAM until process exit. On streaming problems
// with 10-65 MB of input/output (`dmopc21c2p2`), peak RSS jumps 5x-37x over streaming `fread`.
// Fix: replace `cpl::read_all()` with a fixed 64 KB streaming `fread` ring/chunk buffer,
// and auto-flush `cpl::out` when its buffer exceeds 64 KB.

let (n, q): (Int, Int) = read()
var total = 0
for _ in 0..q:
    let (a, b): (Int, Int) = read()
    total += a ^ b
print(total)
