Hand-rolling a binary lifting ancestor table (`up[k, v] = up[k - 1, up[k - 1, v]]`) duplicates `Rooted.lca` / `Rooted.kth_ancestor` and `JumpTree` in `std.tree`.
