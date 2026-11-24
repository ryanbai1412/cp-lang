# Friction report: ucup5s03f

`dream.cp` is verified. The dream reads `a: [0..=m]0..=n` and tallies it into `freq: [0..=n]Int`, but at n, m <= 5000. The port has the statement's n <= 1e4, m <= 2e5, and reads each size straight into `freq: [0..=n]0..=200_001`.

## The tally is a read loop because a whole `[0..=m]` read is not needed
- **Where:** port lines 31-34: `for _ in 0..=m: let x: 0..=n = read(); freq[x] += 1`.
- **What felt wrong:** "count how often each size occurs" is `read<[0..=m]0..=n>().counts()` in one expression. The port keeps the loop so that it never holds the 2e5 sizes, which is a valid memory choice but not one the reader should have to make here.
- **Goal it hurts:** express.
- **Ideal code:** `let freq = read<[0..=m]0..=n>().counts(over: 0..=n)`.
- **General fix:** a std `counts(over:)` histogram into an array over a range or domain (the same as noip22p3's `cities`). Owner: stdlib.
- **Severity:** low.

## Max timing
On an idle machine, at the `--max` case (n = 1e4, m = 2e5), the port runs in 4.79 s against the accepted C++'s 5.63 s, under the 7 s limit. The O(n^2) Krawtchouk stepping dominates in both. The loaded-machine numbers below came from a busy VM.

## Measurements

- **Tokens:** port 212, accepted C++ 2648 (comments excluded).
- **Runtime and memory on gen.py --max:** port 12.72 s / 3 MB, accepted C++ 16.02 s / 3 MB, with the same output.
- **Emitted C++:** 21073 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
