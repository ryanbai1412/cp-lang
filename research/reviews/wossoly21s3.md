# wossoly21s3

Score: 4/5
Dream: 4/5

Cold review: the hand-written queue and DIRS table are gone: each leg is a std bfs over p.adj4.filter_map(land) with until:, and land reads the portal rules once with is_lower/is_upper/lower. The four legs are one list and all_some turns any None into -1.

## Cold review (HEAD)

Port 4/5, dream 4/5 (the dream is now a verified copy of the port). Four std `bfs` legs with `until:`, `land(q)` deciding where a step onto `q` really lands (lines 23-25), and `.all_some()` joining them to `?List<Int>` (line 33): this is the editorial.
- `'0' + k` (line 33) walks the marks by character arithmetic (Owner: idioms, in the friction).
- `place` (lines 18-20) records every cell's character, `.` and `#` included, when only marks and small letters are ever looked up. Harmless, but it reads as more than it means.
- Speed and memory are under the accepted C++ at `--max`.
