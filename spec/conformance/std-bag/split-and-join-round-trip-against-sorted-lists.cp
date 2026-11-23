// expect: ok, output "YES\n"
// SortedBag, SortedSet and SortedMap split_at then join round-trip against sorted lists.
for _ in 0..200:
    let xs = [rand(0, 20) for _ in 0..rand(0, 30)].sorted()
    let k = rand(-1, 21)
    var b = SortedBag.of(xs)
    let hi = b.split_at(k)
    assert b.to_list() == [x for x in xs if x < k] and hi.to_list() == [x for x in xs if x >= k]
    b.join(hi)
    assert b.to_list() == xs and b.len == xs.len
    var lo = SortedBag.of([x for x in xs if x < k])
    lo.join(SortedBag.of([x for x in xs if x >= k]))
    assert lo.to_list() == xs and lo.first == xs.first and lo.last == xs.last
    var s = SortedSet.of(xs)
    let sh = s.split_at(k)
    s.join(sh)
    assert s.len == SortedSet.of(xs).len
    var m: SortedMap<Int, Int> = SortedMap()
    for x in xs: m.set(x, 3 * x)
    let mh = m.split_at(k)
    assert m.len + mh.len == SortedSet.of(xs).len
    for x in xs:
        assert (if x < k: m.get(x) else: mh.get(x)) == Some(3 * x)
        assert (if x < k: mh.get(x) else: m.get(x)) == None
    m.join(mh)
    for x in xs: assert m.get(x) == Some(3 * x)

print("YES")
