# Friction report: coci21c2p3

Rethought. The v0.2 port built an adjacency list of (node, bit) pairs by hand and labelled the nodes with a stack DFS through `?Int` labels. The v0.3 port roots the tree with `Rooted(Graph.undirected(edges))` and labels it in one `tree.down`: a node's label is its parent's XOR the bit of `tree.up_arc(v)!.e`. The even translations are a filtered `for flip in Subset<E> if ..`.

No language gap.

## Measurements

gen.py --max (n = 16): port 0.07 s, accepted C++ 0.08 s, same output.
