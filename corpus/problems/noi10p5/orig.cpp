// NOI 2010 P5 (Route Planning), written for the corpus: no accepted C++ exists. A broken-profile
// DP over the cells in column-major order. Each plug (an edge from a placed cell into an unplaced
// one) carries the path index its unplaced end must have, so a cell's index is read off its incoming
// plugs, or guessed when it has none; L is matched as each index is fixed, and the indices on every
// edge differ by one, which also rules out cycles. A finished profile is one path from index 0, a
// border cell, to index n*m - 1.
#include <bits/stdc++.h>
using namespace std;
typedef unsigned long long u64;
const int MOD = 11192869;

int n, m, N;

// Open-addressing map from profile to ways, cleared between cells.
struct Table {
    vector<u64> keys; vector<int> vals; vector<int> used;
    Table() : keys(1 << 22, ~0ULL), vals(1 << 22) {}
    void add(u64 k, int w) {
        size_t h = (k * 0x9E3779B97F4A7C15ULL) >> 42;
        while (keys[h] != ~0ULL && keys[h] != k) h = (h + 1) & ((1 << 22) - 1);
        if (keys[h] == ~0ULL) keys[h] = k, vals[h] = 0, used.push_back(h);
        vals[h] = (vals[h] + w) % MOD;
    }
    void clear() { for (int h : used) keys[h] = ~0ULL; used.clear(); }
};
int T[3][50], L[150];

// A slot holds 0 (no plug) or 1 + 2 * target + (source is target + 1).
static inline int slot(u64 s, int i) { return s >> (9 * i) & 511; }
static inline u64 with(u64 s, int i, int v) { return (s & ~(511ULL << (9 * i))) | (u64)v << (9 * i); }
static inline int plug(int target, int source) { return 1 + 2 * target + (source > target); }

int main() {
    if (scanf("%d %d", &n, &m) != 2) return 0;
    N = n * m;
    for (int i = 0; i < n; i++)
        for (int j = 0; j < m; j++)
            if (scanf("%d", &T[i][j]) != 1) return 0;
    for (int i = 0; i < N; i++)
        if (scanf("%d", &L[i]) != 1) return 0;
    const int V = n;  // the vertical slot
    static Table A, B;
    Table *cur = &A, *nxt = &B;
    cur->add(0, 1);
    for (int j = 0; j < m; j++)
        for (int r = 0; r < n; r++) {
            nxt->clear();
            bool border = r == 0 || r == n - 1 || j == 0 || j == m - 1;
            for (int h : cur->used) {
                u64 s = cur->keys[h]; int ways = cur->vals[h];
                int in[2] = {slot(s, r), slot(s, V)}, k = 0, t = -1;
                int src[2];
                for (int x : in)
                    if (x) {
                        int tt = (x - 1) / 2;
                        if (t >= 0 && tt != t) goto next_state;
                        t = tt; src[k++] = (x - 1) % 2 ? tt + 1 : tt - 1;
                    }
                if (k == 2 && src[0] == src[1]) continue;
                {
                    u64 base = with(with(s, r, 0), V, 0);
                    int lo = t >= 0 ? t : 0, hi = t >= 0 ? t : N - 1;
                    for (int idx = lo; idx <= hi; idx++) {
                        if (T[r][j] != L[idx] || (idx == 0 && !border)) continue;
                        int need[2], c = 0;
                        for (int u : {idx - 1, idx + 1}) {
                            if (u < 0 || u >= N) continue;
                            bool got = false;
                            for (int q = 0; q < k; q++) got |= src[q] == u;
                            if (!got) need[c++] = u;
                        }
                        if (k + c != (idx > 0) + (idx < N - 1)) continue;
                        bool right = j + 1 < m, down = r + 1 < n;
                        auto add = [&](u64 key) { nxt->add(key, ways); };
                        if (c == 0) add(base);
                        else if (c == 1) {
                            if (right) add(with(base, r, plug(need[0], idx)));
                            if (down) add(with(base, V, plug(need[0], idx)));
                        } else if (right && down) {
                            add(with(with(base, r, plug(need[0], idx)), V, plug(need[1], idx)));
                            add(with(with(base, r, plug(need[1], idx)), V, plug(need[0], idx)));
                        }
                    }
                }
            next_state:;
            }
            swap(cur, nxt);
        }
    int ans = 0;
    for (int h : cur->used) if (cur->keys[h] == 0) ans = cur->vals[h];
    printf("%d\n", ans);
}
