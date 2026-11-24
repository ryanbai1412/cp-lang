# dmpg16g2

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port. The header states the count as inclusion-exclusion over the letters left out, swept bottom row by bottom row, and the body is that sweep. Free runs are a `Dsu<Col>` per mask, and `total[mask]` tracks the pairs inside runs.

This review fixed the port where today's language already allowed the dream's spelling:
- `last` was `[Col, 0..M]-1..n = -1` with `-1` meaning "letter not seen" (R5). It is now `[Col, 0..M]?Row`, and the drops loop reads `if let r = last[c, b]`.
- The neighbour walks `c.ord > 0 and blocked[mask, Col(c.ord - 1)]` and `for d in c.ord - 1 ..= c.ord + 1: if d != c.ord and d in Col` are now `c.prev` and `[c.prev, c.next]`.

The port still agrees with the accepted C++ on 30 gen.py cases and passes its 8 package cases.

## Checklist failures

- Letters are raw `0..M` integers (`Int(ch) - 65`), and a mask's letters come from `(mask.ord >> b) % 2` where the dream has `domain Letter` and `Absent` subsets (R3). The friction reports why: `Letter.subsets` and integer-domain subset lowering fail in the compiler.
- `for top in Row(0)..=bot` names the first row by ordinal; `Row.first!..=bot` says it.

## Gaps that need the language

- The four compiler/diagnostic items in research/frictions/dmpg16g2.md (`Letter.subsets`, subset lowering on integer domains, `buckets()` over `?Row`, enum `.at()`).
