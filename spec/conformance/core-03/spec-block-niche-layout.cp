// expect: ok, output "YES YES"
// rule: 03.2.2
domain V = 1..=4
struct Edge: to: V, w: Int          // 4 + 8 -> 16 B (padded)
var best: [V]?Edge = [V].filled(None)   // 16 B per slot, not 24
var span: ?(lo: Int, hi: Int) = None    // 16 B: None lives in lo
print(yes(best[1] is None), yes(span is None))
