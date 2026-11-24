# sdoi17r1p2

Score: pending
Dream: pending

Follow-up: the enum payloads are now `V` (`Paint(x: V) | Path(x: V, y: V) | Deepest(x: V)`)
since Core's packed-width fix (3670b16c2), so the `V(x)` conversions at each use are gone.
