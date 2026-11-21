// Codeforces 2165 F: count the subarrays that contain a 21435 pattern.
// best[r] is the latest left end of a 21435 ending by r; exchange arguments make "5" the next
// greater after "3", "4" the latest earlier value between them, "1" the next smaller after "2".
@!multitest
domain I = read(1..=1e6)
domain V = 1..=I.count
let a: [I]V = read()

// shapes[k]: ("3" value, "5" position) of each "435" whose "4" is at k;
// every value between a[j] and a[i] sits before j, since i is j's next greater
var shapes: [I]List<(V, I)> = []
var latest = SegTree<V, ?I>(max)
let next = a.next_greater()
for j in I:
    if let i = next[j], let k = latest.fold(a[j] <.. a[i]): shapes[k].push((a[j], i))
    latest[a[j]] = j

// two: the latest "2" per value whose "1" (its next smaller) came before i
var two = SegTree<V, ?I>(max)
var best: [I]?I = none
let freed = a.next_smaller().inverse   // freed[s]: the positions whose next smaller is s
for i in I:
    for (v, r) in shapes[i]: best[r] max= two.fold(..v)
    for j in freed[i]: two[a[j]] = j

print(sum(l.label for l in scan(best, max).present))
