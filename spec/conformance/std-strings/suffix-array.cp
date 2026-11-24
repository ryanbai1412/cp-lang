// expect: ok, output "YES YES YES YES\nYES YES YES\nYES YES YES YES\nYES YES YES YES\nYES YES YES\nYES YES YES"
let b = SuffixArray("banana")
print(yes(b.sa == [5, 3, 1, 0, 4, 2]), yes(b.lcp == [1, 3, 0, 0, 2]), yes(b.len == 6), yes(SuffixArray("").is_empty))
print(yes(b.lcp_between(1, 3) == 3), yes(b.lcp_between(2, 2) == 4), yes(b.lcp_between(0, 6) == 0))
print(yes(b.lcp_range(1, 3) == 1..3), yes(b.compare(1, 3, 3, 5) == 0), yes(b.compare(1, 2, 1, 3) == -1), yes(b.compare(0, 1, 1, 2) == 1))
print(yes(b.distinct() == 15), yes(b.lcp_of_ranks(1..3) == 3), yes(b.lcp_of_ranks(3..4) == 6), yes(b.split(0..3) == 1 and b.split(0..6) == 3))
print(yes(b.kth_distinct(1) == Some(3..5)), yes(b.kth_distinct(15) == None), yes(b.kth_distinct(0) == Some(5..6)))
print(yes(suffix_array([3, 1, 2]).sa == [1, 2, 0]), yes(SuffixArray("aaaa").distinct() == 4), yes(SuffixArray("abcd").kth_distinct(9) == Some(3..4)))
