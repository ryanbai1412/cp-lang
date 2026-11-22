Converting a domain index to `.ord`, adding or subtracting an offset, and wrapping it back in `V(...)` bypasses checked neighbor operations. Use `v.next`, `v.prev`, or `v + k` (which return `?V`).
