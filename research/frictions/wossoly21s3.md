# Friction report: wossoly21s3

`dream.cp` is now a copy of the port (added with its scored review; `corpus.py verify` checks it): the port is the ideal. It adds four std `bfs` legs, where a step onto `q` lands at `land(q)`, with portal letters looked up in a `Map<Char, Cell>`, and joins the legs with `all_some()`.

## Dream

The port as written, with `walk(k, k + 1)` taking mark digits.

## Mark characters come from character arithmetic
- **Where:** line 43: `walk('0' + k, '0' + k + 1) for k in 0..4`.
- **What felt wrong:** the five marks are the characters `'0'..='4'`, but the port adds an offset to `'0'`.
- **Goal it hurts:** express.
- **Ideal code:** `for (from, to) in ('0'..='4').windows(2)`, which iterates characters directly.
- **General fix:** iterating `Char` ranges and taking `windows` on them. If that exists, the port owes the rewrite. Owner: idioms.
- **Severity:** low.

There are no other dream/port gaps.

## Measurements

- **Tokens:** port 224, accepted C++ 800 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.00 s / 3 MB, with the same output.
- **Emitted C++:** 30430 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
