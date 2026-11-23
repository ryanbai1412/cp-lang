# mo: doc-standard notes

- `mo_order`: "sqrt-sized left block" left the block size and origin open, so examples could not be pinned. Chose `b = max(1, isqrt(n))` with `n` = least start to greatest end over the queries, blocks counted from the least start; works for domain and plain `Int` ranges alike (no `D.count` needed).
- `mo_order`: "alternating right-endpoint direction" pinned as even blocks ascending by end, odd blocks descending (block 0 ascending, the common convention).
- `mo_order`, `mo_order_hilbert`, `mo_with_updates`: equal sort keys keep the earlier query first (stable), the library's default tie rule.
- `mo_steps`: the order of the four phases was open beyond "expand before shrinking". Chose left growth, right growth, left shrink, right shrink, each moving one position at a time from the current end; never inverts even for disjoint windows.
- `mo`: the starting window was unspecified. Chose an empty window at the first visited query's start.
- `mo_order_hilbert`: curve orientation was unspecified, which changes the order. Chose the standard `xy2d` Hilbert construction on `(start - lo, end - lo)` over the least `2^k > n` grid.
- `mo_rollback`: pinned the classic schedule (queries inside one block answered alone between `mark`/`undo`; others per block by increasing end, one `reset` per block) and stated the observable contract: when `answer(qi)` runs the state equals a reset structure with `queries[qi]` added.
- `mo_with_updates`: `time` meaning was implicit. Chose "the first `time` updates are applied", trapping outside `0..=updates.len`; block size `max(1, floor(n^(2/3)))`, sort key `(start block, end block, time)` without snaking.
- `mo_rollback` cost: the old prose said "Calls `reset()` once per block" but the classic algorithm only needs it for blocks with a multi-block query; kept "one `reset` per block" as an upper bound.
