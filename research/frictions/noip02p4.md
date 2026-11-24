# Friction Report: noip02p4

## 1. Bounding box is hand-rolled
- **Where:** `type Box`, `grow`, `size`.
- **What felt wrong:** "the bounding box of these points" is a std idea (a `Rect` monoid in geometry), but
  std has no box that grows by points, so the port writes the fold.
- **Goal it hurts:** express.
- **Ideal code:** `Rect.around(ps).area`, and `[0..4]Rect.Empty` per arm with `box[a] += p`.
- **General fix:** a `Rect` bounding-box monoid in `std.geometry` (empty, `+ Pt`, `area`).
- **Severity:** low.

## 2. Accepted C++ handles k <= 3 only
- **Where:** `original:` line.
- **What felt wrong:** the C++ never tries the four-rectangle pinwheel, so it is wrong on k = 4 inputs
  the generator makes; the port is checked against the v0.2 port instead.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port; the k <= 3 AC 5096459 disagrees on k = 4 as expected. Max (n = 50, k = 4) 0.17 s vs the v0.2 port 0.13 s.

## Dream
The ungated ideal (does not check today):

```cp
let (n, k) = read<(1..=50, 1..=4)>()
let pts = read<[n]Pt<0..=500>>().collect()
fn cover(ps: List<Pt>, k: Int) -> Int:
    var best = Rect.around(ps).area
    for turn in [id, Pt.swap]:
        let s = ps.map(turn).sorted()
        for i in 1..s.len if s[i - 1].x < s[i].x:
            for j in 1..k: best min= cover(s[..i], j) + cover(s[i..], k - j)
    if k == 4: best min= min(pinwheel(ps), pinwheel(ps.map(Pt.mirror_x))) ?? best
    best
print(cover(pts, k))
```
