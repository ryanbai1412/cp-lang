# Friction Report: noi20p5

No friction left: the hand-rolled 4-ary trie (flat `kid` list with `4 * o + shape` strides,
`ends` list, `step` that allocates) is std `Trie<Shape>`, with `add` per word, `step` and `ends_at`
for the check; the four shapes are an enum instead of Ints 0..4, a child is `?Node` instead of a
0 sentinel, and the case loop is `@multitest`. A tree that stops at a node whose two children
both branch adds no word: the trie node it used to create could only fail the check where its
missing parent edge already fails it.

Evidence: 30/30 stress against the v0.2 port and 30/30 against the accepted C++; on the max case
the port takes 0.08 s against 0.15 s for the C++, with the same output.
