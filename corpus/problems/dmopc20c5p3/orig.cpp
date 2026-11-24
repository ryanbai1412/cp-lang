// A path exists only if a monotone (U/R) one does (official editorial), so the answer is the
// greedy R-before-U walk through cells that can still reach (N, N) moving up or right. With at
// most one block per row, the cells of row r that cannot (dead, blocks included) are at most two
// intervals: one ending at column N and one ending at row r's block; sweep rows top-down.
#include <bits/stdc++.h>
using namespace std;
struct Seg {
    int lo = 1, hi = 0;
    bool has(int c) const { return lo <= c && c <= hi; }
};
int main() {
    int n, k;
    scanf("%d %d", &n, &k);
    vector<int> blk(n + 2, 0);
    for (int i = 0; i < k; i++) {
        int r, c;
        scanf("%d %d", &r, &c);
        blk[r] = c;
    }
    vector<Seg> tail(n + 2), atb(n + 2);
    if (blk[n]) atb[n] = {1, blk[n]};
    for (int r = n - 1; r >= 1; r--) {
        Seg up_tail = tail[r + 1], up_b = atb[r + 1];
        int b = blk[r];
        Seg t, s;
        if (up_tail.lo <= up_tail.hi) t = up_tail;
        if (b) {
            int lo = b;
            if (up_tail.has(b - 1)) lo = up_tail.lo;
            else if (up_b.has(b - 1)) lo = up_b.lo;
            if (t.has(b)) {
                lo = min(lo, t.lo);
                t = {b + 1, n};
                if (t.lo > t.hi) t = Seg{};
            }
            s = {lo, b};
            if (b == n) t = s, s = Seg{};
            else if (t.lo <= t.hi && t.lo == b + 1) t.lo = s.lo, s = Seg{};
        }
        tail[r] = t, atb[r] = s;
    }
    auto dead = [&](int r, int c) { return tail[r].has(c) || atb[r].has(c); };
    if (dead(1, 1)) {
        puts("IMPOSSIBLE");
        return 0;
    }
    string out;
    out.reserve(2 * n);
    int r = 1, c = 1;
    while (r != n || c != n) {
        if (c < n && !dead(r, c + 1)) out += 'R', c++;
        else out += 'U', r++;
    }
    puts(out.c_str());
}
