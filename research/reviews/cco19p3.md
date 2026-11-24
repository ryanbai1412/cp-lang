# cco19p3

Score: 4/5
Dream: 4/5

The hub argument is the program: `Rooted.from_parents`, `up` for the people under each city, `down` to reroot `chain`, and std `subset_sums` meet-in-the-middle for the cross term. 0 hand tree walks left.

## Checklist failures

None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- `from_parents` wants `[V]?V`, so the `[Kid]V` read is widened by `Kid.at(Int(v))`: research/frictions/cco19p3.md.
- `down`'s step sees only the child, so the root's `None` arm is written out: same report.

## Dream notes

- The three-line dream is the port with the two gaps closed; `a[t.parent(v)!]` relies on `down` never stepping into the root.
