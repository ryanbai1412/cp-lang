# Friction report: noip22p4

There is no dream.cp; the ideal is below. The port is the editorial's offline sweep by right end. A lazy `SegTree` with `Action<Sums, Tag>` keeps A, B, A·B and the history sum. `a.spans(by: x => -x)` gives each value's reign, and `buckets()` groups the queries by right end. The `U64` wraparound is the requested mod 2^64.

## Dream

```cp
struct Tag(set_a: ?U64 = None, set_b: ?U64 = None, ab: U64 = 0, a: U64 = 0, b: U64 = 0, one: U64 = 0)
let set_a = (x: U64) => Tag(set_a: Some(x))
let record = Tag(ab: 1)
```

## Tags with mostly zero fields are written out in full
- **Where:** lines 64-67: `id: Tag(None, None, U64(0), U64(0), U64(0), U64(0))`, `set_a`, `set_b` and `record`, each with six positional fields, plus `none = Sums(U64(0), ...)`.
- **What felt wrong:** each tag differs from the identity in one field. Positional zeros hide which field is set, and adding a field means editing every constructor.
- **Goal it hurts:** express, revision.
- **Ideal code:** the dream's field defaults with named arguments.
- **General fix:** default values for struct fields, so that a constructor call names only the fields it changes. Owner: core/spec.
- **Severity:** low.

## Measurements

- **Tokens:** port 884, accepted C++ 2503 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.87 s / 61 MB, accepted C++ 2.57 s / 169 MB, with the same output.
- **Emitted C++:** 28260 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
