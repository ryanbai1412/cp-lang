// expect: ok, output "5 7 4 2"
// rule: 06.3.4 07.6.3
var seen = 0
for s in [1, 2, 3].subsequences():
    seen += 1
    if s.items == [1]: s.prune()

var visits = 0
let kept = [1, 2, 3].subsequences().filter(s => { visits += 1; s.items.len <= 1 or { s.prune(); false } }).count(s => true)

struct Step(v: Int, pruned: Bool = false):
    fn prune(mut self):
        pruned = true

fn walk() -> yields Step:
    for i in 1..=5:
        var st = Step(i, false)
        yield st
        if st.pruned: return

var custom = 0
for st in walk():
    custom += 1
    if st.v == 2: st.prune()

print(seen, visits, kept, custom)
