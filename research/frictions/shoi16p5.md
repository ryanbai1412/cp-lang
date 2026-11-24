# Friction report: shoi16p5

There is no dream.cp. The port is the ideal:
- each position is the std `Affine` map `x => a_i x + a_i closes[i]`;
- a `SegTree` of their compositions keeps the whole sum;
- an update is `chain.set` and the answer is `chain.all.b`.

## Dream

The port as written.

There are no dream/port gaps. The reversed `Affine.then(g, f)` in the monoid (each position's map nests inside the previous one) is stated in the comment above it.

## Measurements

- **Tokens:** port 218, accepted C++ 1166 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.06 s / 6 MB, accepted C++ 0.05 s / 8 MB, with the same output.
- **Emitted C++:** 30550 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
