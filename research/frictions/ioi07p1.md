# Friction Report: ioi07p1

## Dream

```cp
domain Cell = 1..=n
fn examine(x: Cell?, y: Cell?) -> Bool   // off the meadow is plain `false`
fn reach(dx: Int, dy: Int) -> Int = gallop_first(1, k => not examine(x0 + dx * k, y0 + dy * k))! - 1
```

## Gaps

- **`ask` needs a typed `let` to read its reply.** `ask("examine", x, y) == "true"`
  can't infer the reply type, so `examine` binds `rep: Str` first. Severity: low.
- **The hops to the lower-left square are three hand `while` loops.** Each is
  "step while still flattened", a `gallop`-shaped walk that std has no
  stepping form for. Severity: low.
