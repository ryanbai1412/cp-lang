# Friction Report: ioi08p4

## Dream

```cp
let ride = Functional(stretch_after).walk(from: 0)        // the stretches the ride crosses
let loops = Functional(stretch_after).cycles().map(.len)  // the loops it never visits
```

## Gaps

- **The ride is a path, not a cycle, so `Functional` doesn't fit.** Stretch
  `2n` (east of the last endpoint) has no successor, and a functional graph
  needs one per vertex. So the port walks the ride and then each unseen loop
  by hand, with a shared `seen`. Severity: low.
- **Numbering the endpoints west to east goes through buckets.** `owner:
  [Spot]?Tele` and `first_end` rank the 2n endpoints. A coordinate-compression
  domain (`domain Point = ranks of teles.w, teles.e`) would give the
  numbering directly. Severity: low.
