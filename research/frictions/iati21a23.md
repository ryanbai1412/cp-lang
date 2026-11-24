# Friction Report: iati21a23

## Dream

```cp
var alive = IntervalSet(1..=n)        // the cells the target may be in
for t in 0..: shoot(the end it moves toward); alive = alive.step(fleeing: t % period < s)
```

## Gaps

- **The windows are hand-moved integer ends.** Each turn edits `l` and `r`
  by hand, with a width-2 special case; a window type with `grow`/`shrink`
  clamped to `1..=n` would carry the bounds. Severity: low.
