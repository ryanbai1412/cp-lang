# ucrpc21d review

Score: pending
Dream: pending

Cold review: rethought. The old port flood-filled canvas cells with a hand stack and a four-way
`doors` helper. The rooms are now their own `domain Room = (Row, Col)`, not canvas
coordinates: one pass joins each room to its right and lower neighbour in a std `Dsu<Room>`
when the character between them is `.`, and the answer is the four largest
`regions.size(root)`, sorted with `desc`. The canvas is read typed as
`[0..=2 * r, 0..=2 * c]Char`.
