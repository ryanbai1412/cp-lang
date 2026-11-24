# Friction report: noi02p6

## Dream

The port is the dream: `Mod<10_000>` arithmetic, a fold over the factors, `prod` for m.

## Hand modular arithmetic becomes `Mod`

The v0.2 port wrapped everything in a `main_solve` with a hand `mpow`, `% modv` after every
step and `+ 2 * modv` before subtraction. The rewrite uses `type Z = Mod<10_000>`, a tuple
update for the even/odd sums, and `prod(Z(f.p) ** f.e ...)`. Input bounds are in the read
type.

## Verification

30 random cases agree with the v0.2 port and 30 with `orig.cpp`. Corpus tests pass 8/8.
