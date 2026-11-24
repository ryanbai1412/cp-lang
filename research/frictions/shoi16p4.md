# Friction report: shoi16p4

There is no dream.cp; the ideal is below. The port uses inclusion-exclusion over the skipped companies (`Subset<Company>`). Each set builds the Laplacian minor over `Rest = 1..n` and takes `Mat.new(lap).det()` mod p. It runs in 1.30 s against the accepted C++'s 1.34 s and the 2 s limit.

## Dream

```cp
for skipped in Subset<Company>:
    let edges = Company.filter(c => c not in skipped).flat_map(c => roads[c])
    total += sign(skipped) * spanning_tree_count<City, Z>(edges)
```

## The matrix-tree theorem is written out per port
- **Where:** lines 23-31: the Laplacian is filled edge by edge into `[Rest, Rest]Z`. Each city is mapped to the minor with `Rest.at(u.label)`, and three `if let` updates leave out row and column n.
- **What felt wrong:** "count spanning trees" is a std-level algorithm. The port re-derives Kirchhoff's minor, including which vertex is dropped and how each end maps into `Rest`.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** the dream's `spanning_tree_count<V, Z>(edges)`.
- **General fix:** a std `spanning_tree_count` over a field (Laplacian minor plus `det`). Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 235, accepted C++ 1159 (comments excluded).
- **Runtime and memory on gen.py --max:** port 1.30 s / 3 MB, accepted C++ 1.34 s / 3 MB, with the same output.
- **Emitted C++:** 26412 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
