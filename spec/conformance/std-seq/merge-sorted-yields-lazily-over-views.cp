// expect: ok, output "0 1 2 3 4 5 6 7 8 9\nYES NO\n3 2\n10"
// rule: 08.1.2
// `merge_sorted` yields as it walks: two `.map` views of ranges merge with no list behind
// them, the merge compares with a `List` directly, and a `break` stops reading both inputs.
let evens = (0..5).map(i => 2 * i)
let odds = (0..5).rev.map(i => 9 - 2 * i)
print(evens.merge_sorted(odds))
print(yes(evens.merge_sorted(odds) == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]), yes(merge_sorted(evens, odds) == [0, 1]))
let a = (0..1000).map(i => i * 3)
let b = (0..1000).map(i => i * 3 + 1)
var took = 0
for x in a.merge_sorted(b, by: x => x):
    if x > 3: break
    took += 1
print(took, merge_sorted([5], [1, 9], by: x => -x).collect().len - 1)
print(evens.merge_sorted(odds).count())
