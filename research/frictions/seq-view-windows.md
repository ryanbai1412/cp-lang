# Friction report: `std.seq.windows` / `chunks` allocate `List<T>` per yielded slice

Found during the `std.seq` copy/view audit (`ebbf9f6c`). Signatures were intentionally left unchanged pending a core lending/generator design for yielding lent views.

## `windows` and `chunks` materialize a fresh `List<T>` per yielded window/chunk

- **Where:** `stdlib/seq.cp` (`windows`, `chunks`, `chunk_by`).
- **What felt wrong:** `xs.windows(k)` and `xs.chunks(k)` allocate and copy a fresh `List<T>` of length `k` on every iteration (`O(k)` per step, `O(n * k)` total for `windows(k)`), even when the caller only reads elements inside the `for w in xs.windows(k):` loop. Because generators cannot currently yield a lent slice view (`yields Slice<T>` / `ListView<T>` tied to the `self` borrow), stdlib helpers must allocate owned `List<T>` values.
- **Goal it hurts:** performance
- **Ideal code:** `for w in xs.windows(k):` yields an O(1) read-only slice view borrowing `xs` for the loop body (zero allocation, `O(1)` per step), with `.to_list()` when an owned copy is actually needed.
- **General fix:** allow `yields` generators whose parameter is lent to yield borrowed slice views (borrowing the input sequence for the duration of the loop body, under the existing lending rules so mutating `xs` inside the loop is a compile error), then migrate `windows`, `chunks`, and `chunk_by` (or provide view-yielding counterparts if breaking callers that store yielded `List<T>` is undesirable).
- **Severity:** medium
- **Status:** open
- **Area:** seq
