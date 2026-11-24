# shoi16p3

Score: 4/5
Dream: 4/5

Cold review: reads carry the statement bounds (k < n, top <= 1e9, rank <= n) and the students and courses keep the statement's n and m names; the DP is a typed [Others] table rebuilt by one comprehension per course, so the next/still swap and the List.filled sentinel row are gone.

## Cold review (HEAD)

Port 4/5, dream 4/5 (the port is the dream's body). Each course's count is a polynomial in its top score, evaluated by std `lagrange_at` on n + 2 samples (lines 24-29). Across courses, one `[Others]Z` vector counts the others still never above B (lines 31-40).
- `Z(Int(j.label == n - 1))` (line 31) turns a Bool into a count through `Int`; `if j.label == n - 1: 1 else: 0` says it.
- fmt splits the second `comb.choose(` over four lines inside the `sum` (lines 35-39). Binding `let above = n - 1 - j.label` would keep it on one line.
