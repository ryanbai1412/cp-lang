# noip20p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

## Checklist failures

- `after = List(odd_prefixes(letters.rev).rev)` builds the suffix F with two reversals and a `List` rebuild.
- `for d in before[w - 1]..=Odd.last!: at_most[d] += 1` is a suffix range-add, done one cell at a time inside the main loop (O(26) per w, which is fine, but it is a hand-kept prefix count).
- `copies`, `(copies + 1) / 2` and `copies / 2` count odd and even repetitions by integer division with no names for the two cases.
- `z[w]`, `before[w - 1]` and `after[2 * w]` index positions as bare `Int`s.

## Gaps that need the language

- Suffix F's two reversals: research/frictions/noip20p2.md.
