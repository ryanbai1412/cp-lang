// Exhaustive brute check of Range<Int>.sum() and (Range<Int>).step(k).sum().
for lo in -6..=6:
    for hi in -6..=6:
        var want = 0
        for x in lo..hi: want += x
        assert (lo..hi).sum() == want
        assert sum(lo..hi) == want
        let r = lo..hi
        assert r.sum() == want

        var want_inc = 0
        for x in lo..=hi: want_inc += x
        assert (lo..=hi).sum() == want_inc

        for k in 1..=5:
            var w = 0
            for x in (lo..hi).step(k): w += x
            assert (lo..hi).step(k).sum() == w
            assert sum((lo..hi).step(k)) == w
            var wi = 0
            for x in (lo..=hi).step(k): wi += x
            assert (lo..=hi).step(k).sum() == wi

// O(1) proof: 1e9 elements must complete instantaneously in both interp and C++.
print((1..=1_000_000_000).sum())
print((2..1_000_000_000).step(2).sum())
print((3..1_000_000_000).step(2).sum())
print((5..2).step(2).sum())

// Prefix.sum accepts any Range<D>, including empty and reversed ranges.
domain Pos = 1..=5
let a: [Pos]Int = [Pos] i => i.ord * 10
let pref = prefix(a)
assert pref.fold(Pos(3)..Pos(3)) == 0
assert pref.fold(Pos(4)..Pos(2)) == 0
assert pref.fold(Pos(2)..=Pos(4)) == 60
print(pref.fold(Pos(3)..Pos(3)), pref.fold(Pos(4)..Pos(2)), pref.fold(Pos(2)..=Pos(4)))
