// expect: ok, output "YES YES YES YES\nYES YES YES\nYES YES YES YES\nYES YES YES YES\nYES YES YES\nYES YES YES"
use std.strings.{SuffixArray, suffix_array}
let b = SuffixArray("banana")
print(b.sa == [5, 3, 1, 0, 4, 2], b.lcp == [1, 3, 0, 0, 2], b.len == 6, SuffixArray("").is_empty)
print(b.lcp_between(1, 3) == 3, b.lcp_between(2, 2) == 4, b.lcp_between(0, 6) == 0)
print(b.lcp_range(1, 3) == 1..3, b.compare(1, 3, 3, 5) == 0, b.compare(1, 2, 1, 3) == -1, b.compare(0, 1, 1, 2) == 1)
print(b.distinct() == 15, b.lcp_of_ranks(1..3) == 3, b.lcp_of_ranks(3..4) == 6, b.split(0..3) == 1 and b.split(0..6) == 3)
print(b.kth_distinct(1) == Some(3..5), b.kth_distinct(15) == None, b.kth_distinct(0) == Some(5..6))
print(suffix_array([3, 1, 2]).sa == [1, 2, 0], SuffixArray("aaaa").distinct() == 4, SuffixArray("abcd").kth_distinct(9) == Some(3..4))
