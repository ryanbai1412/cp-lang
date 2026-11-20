// `.rev()` walks backwards exactly as `.rev` does, on every kind of sequence.
for i in (0..3).rev(): print(i)
for i in (0..=3).rev(): print(i)
for i in (0..3).rev: print(i)
let xs = [1, 2, 3]
for x in xs.rev(): print(x)
for x in xs.rev: print(x)
print([i for i in (0..3).rev()], [i for i in (0..3).rev])
for i in (0..10).step(3): print(i)
for i in (0..10).step(3).rev: print(i)
for i in (0..10).step(3).rev(): print(i)
print(sum(i * 10 ** k for (k, i) in [(0, 1), (1, 2)]), xs.rev(), xs.rev, "abc".rev(), "abc".rev)
let n = 5
domain V = 1..=n
var a: [V]Int = [V] v => v.to_idx
for v in V.rev(): print(v)
for x in a.rev(): print(x)
for i in (0..n).rev().rev(): print(i)
for i in (0..n).rev() if i % 2 == 0: print(i)
for c in "abc".rev(): print(c)
var s: Set<Int> = Set()
s.add(1)
s.add(2)
for x in s.rev(): print(x)
print(sum(i for i in (0..4).rev()), [x for x in xs.rev() if x > 1])
