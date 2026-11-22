# Friction Report: noi09p4

## 1. Product domain `(Row, Col)` as a single index domain

- **Area:** core
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_15/dream/noi09p4.cp:17-27`
- **Port:** `proto/ports/scout_batch_15/noi09p4.cp:18-32`

`domain Cell = (Row, Col)` cannot yet be used as a single generic domain parameter to `[Cell]T`, `peel<Cell>`, and `max_closure`. The active port flattens `(r, c)` by hand via `domain Cell = 0..n * m` and `Cell.at(r.ord * m + c.ord)`.

## 2. Length-prefixed list in typed `read()`

- **Area:** core
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_15/dream/noi09p4.cp:25-27`
- **Port:** `proto/ports/scout_batch_15/noi09p4.cp:27-31`

Reading `(value, targets): (Int, List<Cell>) = read()` where the count `k` immediately precedes `k` elements is not supported; the active port reads `k` and loops `0..k`.
