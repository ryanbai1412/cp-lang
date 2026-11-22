// expect: ok, output "YES\nYES\nYES\nYES YES\nYES\nYES YES\nYES YES YES\nYES YES 2\nYES YES\nYES YES\nYES\nYES\nYES"
use std.seq.{sorted_union, sorted_intersect, sorted_diff, kway_merge}
print([1, 2].with_next().collect() == [(1, Some(2)), (2, None)])
var a = [1, 2]
print(a.take_all() == [1, 2] and a == [])
print([1, 4, 2, 5].rfind(x => x % 2 == 0) == Some(2))
print([1, 2, 3].resized(5, 0) == [1, 2, 3, 0, 0], [1, 2, 3].resized(2, 0) == [1, 2])
var s = [1, 4, 5]
print(s.pop_while(x => x > 3) == [5, 4] and s == [1])
var t = [1, 5]
print(t.pop_if(x => x > 3) == Some(5), t.pop_if(x => x > 3) == None)
print(sorted_union([1, 3], [2, 3]) == [1, 2, 3], sorted_intersect([1, 2, 2, 3], [2, 2, 4]) == [2, 2], sorted_diff([1, 2, 2, 3], [2]) == [1, 2, 3])
print([5, 1, 4, 2].min_k(2).sorted() == [1, 2], [5, 1, 4, 2].max_k(2).sorted() == [4, 5], [5, 1].min_k(7).len)
print([1, 2, 1].min_count() == Some((val: 1, count: 2)), [3, 1, 3].max_count() == Some((val: 3, count: 2)))
print([2, 1, 2].majority() == Some((val: 2, count: 2)), [1, 2].majority() == None)
var b = [1, 2, 3, 4]
print(b.partition_in_place(x => x % 2 == 0) == 2)
var c = [1, 2, 3, 4]
print(c.stable_partition_in_place(x => x % 2 == 0) == 2 and c == [2, 4, 1, 3])
print(kway_merge([[1, 4], [2], [3]]) == [1, 2, 3, 4])
