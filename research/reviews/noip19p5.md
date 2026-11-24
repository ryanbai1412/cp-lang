# noip19p5

Score: pending
Dream: pending

Cold review: the deque of candidate cuts is a std Deque<Cut<Pos>>, the predecessor table is [Pos.cuts]?Cut<Pos> with no 0 sentinel, and the prefix sums fill a Prefix<Pos, Int> in place so the 4e7 generator case stays within 1024 MB. The generator recurrence threads (b, after) by hand: Gen<_> is not Indexed, so it cannot be zipped with Pos.
