# Friction report: noip10p1

The port is now the dream, with the statement bounds as read types (`m: 1..=100`, `n: 1..=1000`, words `0..=1000`) and a `// Bounds:` header. The v0.2 port kept a `Set` plus a `List` with a hand `head` index as its queue.

## Dream

`dream.cp`, which the port now matches: a `Deque<Int>` memory with `contains`, `push` and `pop_front`.

There are no dream/port gaps. `memory.contains(w)` is O(M), which at M <= 100 and N <= 1000 is 1e5 steps.

## Measurements

- **Tokens:** port 83, accepted C++ 254 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.00 s / 3 MB, with the same output.
- **Emitted C++:** 12774 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
