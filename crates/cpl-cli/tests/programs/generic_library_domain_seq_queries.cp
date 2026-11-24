struct Tw<W>(pub a: Int)
impl Index for Tw<W>:
    let count = W.count
    @get fn ord(self) -> Int = a
    fn nth(k: Int) -> Tw<W> = Tw(k)
domain V = 0..3
print(Tw<V>.count(x => x.a > 1), Tw<V>.find(x => x.a > 0).map(x => x.a) ?? -1)
print(Tw<V>.filter(x => x.a > 0).count(), Tw<V>.min(by: x => -x.a)!.a)
