// Source: DMOPC '21 Contest 9 P2 (String Puzzle), https://dmoj.ca/problem/dmopc21c9p2
// Problem: T puzzles (sum of lengths <= 1e6), each two lowercase strings A and B. One move
//   replaces two adjacent equal letters by one copy of the next letter ("zz" cannot merge).
//   Print YES if A can be turned into B, else NO.
// Solution: each letter of B is merged from a contiguous block of A. Scan A pushing
//   letters onto a deque and merging the last two while they are equal and not z; whenever
//   the front equals the next unmatched letter of B, pop it and advance. YES iff the deque
//   empties with all of B matched. O(|A| + |B|) amortised.
// original: corpus/submissions/4563973.cpp

@!multitest
let a: Str = read()
let b: Str = read()
var pending = Deque<Char>.new()
var matched = 0
for c in a:
    pending.push(c)
    while pending.len >= 2 and pending.last! == pending[pending.len - 2] and pending.last! != 'z':
        let x = pending.pop()!
        pending.pop()
        pending.push(Char(Int(x) + 1))
    while matched < b.len and pending.first == Some(b[matched]):
        pending.pop_front()
        matched += 1
print(yes(pending.is_empty and matched == b.len))
