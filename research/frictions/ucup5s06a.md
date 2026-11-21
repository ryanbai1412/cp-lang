# ucup5s06a

- **Source:** Universal Cup 5 Stage 6 A (`proto/ports/ecnerwala/ucup5s06a_orig.cpp`)
- **Port:** `proto/ports/ecnerwala/ucup5s06a.cp`
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s06a.cp`
- **Status:** Ported (`gate.sh ucup5s06a 150` passes)

## Power series are bare lists with free functions

- **Goal it hurts:** express
- **Severity:** medium
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s06a.cp:15-18`
- **Actual:** `proto/ports/ecnerwala/ucup5s06a.cp:16-22`
- **General fix:** a truncated `Series<Z>` value in `std.poly` with `+`, `-`, `*`, `/` (by inverse), `deriv`, `x` and a scalar `1`, each truncating to the shorter length. The port spells `1 - B` as `[Z(Int(i == 0)) - b[i] ...]`, `B + x` as a second comprehension, and `[x^n] x B' / (1 - B)` as `inv` plus a hand convolution at one coefficient.

## Literal integers do not lift into `Mod`

- **Goal it hurts:** express
- **Severity:** low
- **Area:** types
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s06a.cp:16-18`
- **Actual:** `proto/ports/ecnerwala/ucup5s06a.cp:20-22`
- **General fix:** an integer literal where a `Mod<P>` is expected is that residue, so `else: 0` and `2 * cyc(b)` need no `Z(0)` or `Z(2)`.

## A bound that depends on an earlier field of the same read

- **Goal it hurts:** hard to get wrong
- **Severity:** low
- **Area:** types
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s06a.cp:14`
- **Actual:** `proto/ports/ecnerwala/ucup5s06a.cp:14`
- **General fix:** tuple reads whose later field type names an earlier field (`(n, k): (Int, 1..n)`), the same dependent bound `read<[0..n]...>` already allows across statements. The statement's limits are not in the source, so `n` stays `Int`.
