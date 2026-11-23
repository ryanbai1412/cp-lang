# noip09p4 friction

## `Subset.complement()` and `s.items` are documented but unknown to `cpl run`
- **Where:** `spec.cp` `free`: `Subset<Digit>.full - used`, and `for d in free(p)`.
- **What felt wrong:** `used.complement()` is E0203 and `for d in s.items` is E0302 "can't walk
  over a `Items<Subset<Digit>>`", though both are in `std/subset.cp`.
- **Goal it hurts:** express, writable from the docs.
- **General fix:** already routed (compiler worklog 2026-11-22, stdlib worklog 2026-11-23).
- **Severity:** low (`full - s` and `for d in s` say the same).

## `s.set(b, on)` inside a fn is silently dropped
- **Where:** the old `place(p, d, on)` with `in_box[box(p)].set(d, on)`.
- **What felt wrong:** a `Subset` mutated by `.set` from inside a fn keeps its old value in
  `cpl run` and in the C++, with no warning, so the box check vanished and the search printed
  boards with a repeated digit in a box (2868 where the answer is -1; tests/09).
- **Goal it hurts:** hard to get wrong (P0 mutation surprise), interp/C++ agree on a wrong answer.
- **Repro:** `domain B = 0..4` / `var x = Subset<B>.empty` / `fn put():` / `    x.set(B(1), true)`
  / `put()` / `print(x.len)` prints 0, want 1. Routed to compiler.
- **Severity:** P0.

## Compound `^=` on a `Subset` element needs a plain-variable index
- **Where:** `toggle`: `let ((r, c), b, bit) = (p, box(p), Subset<Digit>.single(d))`.
- **What felt wrong:** `in_row[p.0] ^= bit` and `in_box[box(p)] ^= bit` are E0309 "`^=` can't
  be applied to a `Subset<Digit>`"; `in_row[r] ^= bit` checks, and `n[p.0] += 1` on `Int` does.
- **Ideal code:** `in_row[p.0] ^= bit; in_col[p.1] ^= bit; in_box[box(p)] ^= bit`.
- **General fix:** type a compound assignment's place once, whatever its index expression.
  Routed to compiler.
- **Severity:** low.

## `Subset.single(d)` needs its type argument
- **What felt wrong:** `s ^ Subset.single(d)` is E0302 "expected `Subset<Digit>`, found
  `Subset<Digit>`"; `Subset<Digit>.single(d)` works. Routed to compiler.
- **Severity:** low.
