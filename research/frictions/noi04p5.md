# Friction report: noi04p5

## Dream

```cp
let (k1, k2): (Decimal<1>, Decimal<1>) = read()      // exact tenths
```

## Already near the bar

The DP over (north patches, width, south patches) with `?Int` and `min=` was already v0.3
style. The rewrite tuple-reads `(n, m)` and puts the multi-line index on one line. The slopes
are still read as `F64` and rounded to tenths. A fixed-point decimal read would make that
exact by type.

## Verification

The frozen v0.2 snapshot reads the slopes with an older `read_tenths` input format and can't
parse the current generator. The port therefore matches the previous main port (30 cases) and
the accepted C++ (30 cases) instead. Corpus tests pass.
