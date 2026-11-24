# Friction report: noi07p2

## Dream

```cp
var lines = LiChao(max, over: by_ratio)        // keyed by the sorted ratios themselves
```

## Hand Li Chao tree becomes std `LiChao`

The v0.2 port kept its own Li Chao tree over `List<?Line>` with `4 * n` cells, plus
hand-written add/highest loops, and read the three day arrays field by field. The rewrite
reads typed `days` and uses std `LiChao<Worth, F64>(max, over: 0..n)` over the ratio ranks. A
custom `Worth` line evaluates `k * ratio[x] + m`, so no two reals are compared for equality.
`LiChao` coordinates are `Int`, so the ranks stand in for the real ratios.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. The max case runs in
0.08 s. Corpus tests pass.
