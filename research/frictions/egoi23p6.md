# Friction report: egoi23p6

Rethought. The v0.2 port sized the swap budget at `n * n / 2 + 10`, wrapped every index in `Take(..)`, `Pos(..)` or `Swaps(..)` domain constructors, and found the answer with a `?Int` loop. The v0.3 port bounds the budget exactly: the dearest pick moves the last f boxes, f (n - f) swaps. Its three dimensions are a slot count, a box position and a swap budget, each used in arithmetic (`j + 1`, `s - (j - i)`), so they are typed ranges `[0..=f, 0..=n, 0..=cap]`. The answer is `(0..=cap).find(..)` under `if let`. The table is half the size.

No language gap.

## Measurements

gen.py --max (n = 100, f = 50): port 0.11 s / 104 MB, v0.2 port 0.12 s / 204 MB. The accepted C++ keeps its table on the stack and crashes under an 8 MB stack; with an unlimited stack it gives the same output.
