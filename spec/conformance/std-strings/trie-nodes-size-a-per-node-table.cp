// expect: ok, output "6 4 2"
// A per-node table is sized by `nodes`, not `len`, which counts stored words.
use std.strings.{Trie}
var t = Trie<Char>()
var hits: List<Int> = []
for w in ["ba", "a", "ba", "abc"]:
    let at = t.add(w)
    hits.resize(t.nodes, 0)
    hits[at] += 1
print(t.nodes, t.len, hits[t.find("ba")!])
