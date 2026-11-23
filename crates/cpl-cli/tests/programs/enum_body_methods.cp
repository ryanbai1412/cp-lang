enum Ev:
    Add(v: Int)
    Ask(l: Int, r: Int)
    Stop
    fn is_query(self) -> Bool = self is Ask
    fn weight(self) -> Int:
        match self:
            Add(v) => v
            Ask => 0
            Stop => -1
let evs = [Add(5), Ask(1, 2), Stop]
for e in evs: print(if e.is_query(): 1 else: 0, e.weight())
