# Friction Report: ceoi22p1

## 1. Non-typeable input constraints (`Perm<Card>` and even `n`)

- **Area:** core
- **Goal:** hard-to-get-wrong
- **Severity:** low
- **Dream:** `proto/ports/scout_batch_24/dream/ceoi22p1.cp:18-21`
- **Port:** `proto/ports/scout_batch_24/ceoi22p1.cp:18-21`

The statement guarantees `n` is even and `deck` is a permutation of `1..=n`, neither of which can be declared on `read()` so `cpl stress` needs a custom generator.

## 2. Domain range length (`(p..=end_p).len`) without `.ord`

- **Area:** ranges
- **Goal:** express
- **Severity:** low
- **Dream:** `proto/ports/scout_batch_24/dream/ceoi22p1.cp:36,64`
- **Port:** `proto/ports/scout_batch_24/ceoi22p1.cp:36,64`

Computing a run's length (`end_p.ord - p.ord + 1`) and 1-based query rank (`qs[qi].i.ord + 1`) still steps through `.ord` instead of `(p..=end_p).len`.
