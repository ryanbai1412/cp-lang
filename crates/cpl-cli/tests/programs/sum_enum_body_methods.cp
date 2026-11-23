domain V = 1..=4
enum Ev:
    Add(v: V)
    Ask(l: V, r: V)
    fn is_query(self) -> Bool = self is Ask
    fn weight(self) -> Int:
        match self:
            Add(v) => v.idx
            Ask => -1
let evs = [Add(3), Ask(1, 2)]
for e in evs: print(if e.is_query(): 1 else: 0, e.weight())
if Add(4) is Ask: print(9)
