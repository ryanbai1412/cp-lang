// IATI 2021 Heaps. No accepted submission exists (the C++ ones are WA, RTE or TLE). A move
// on heap (B, S) that removes stones may leave any count of pebbles from K X up, so heap
// values are ordinals below w^2; write w c + d as (c, d). The sum is lost exactly when the
// c's and the d's both XOR to 0 (ordinal nim-sum). The options of (b, s) that keep b are its
// smaller s, and the rest form a set R_b independent of s, so g(b, .) lists the ordinals
// outside R_b in increasing order. Two facts, checked against a brute of the definition
// for K <= 4, b <= 24, s <= 30: g(b, s) = (b, s - K b) for s >= K b, and below that
// g(b, s) = (c, d K + s mod K) where (c, d) = f(b, s div K), f being the K = 1 table. For
// K = 1, R_b holds level c from d = max(0, b - 2c) up plus f(b', t) for b - b' <= t < b', so
// f(b, .) lists, level by level, the d below that start that no such f(b', t) covers.
#include <bits/stdc++.h>
using namespace std;

int main() {
    int k, q;
    if (scanf("%d %d", &k, &q) != 2) return 1;
    vector<vector<pair<int, int>>> tests(q);
    int maxb = 0;
    for (auto& t : tests) {
        int n;
        if (scanf("%d", &n) != 1) return 1;
        t.resize(n);
        for (auto& [b, s] : t) {
            if (scanf("%d %d", &b, &s) != 2) return 1;
            maxb = max(maxb, b);
        }
    }
    vector<vector<pair<short, short>>> f(maxb + 1);
    vector<vector<int>> mark(maxb + 1);
    vector<int> stamp(maxb + 1, 0);
    for (int b = 1; b <= maxb; b++) {
        // mark[c][d] == b: (c, d) with d < b - 2c is reached from a lower row
        for (int bp = b / 2 + 1; bp < b; bp++)
            for (int t = b - bp; t < bp; t++) {
                auto [c, d] = f[bp][t];
                if (d < b - 2 * c) {
                    auto& row = mark[c];
                    if ((int)row.size() <= d) row.resize(max<int>(d + 1, 2 * row.size()), 0);
                    row[d] = b;
                }
            }
        for (int c = 0; 2 * c < b && (int)f[b].size() < b; c++)
            for (int d = 0; d < b - 2 * c && (int)f[b].size() < b; d++)
                if ((int)mark[c].size() <= d || mark[c][d] != b) f[b].push_back({(short)c, (short)d});
        if ((int)f[b].size() != b) return 2;
    }
    for (auto& t : tests) {
        long long xc = 0, xd = 0;
        for (auto [b, s] : t) {
            long long c, d;
            if ((long long)s >= (long long)k * b) c = b, d = s - (long long)k * b;
            else {
                auto [fc, fd] = f[b][s / k];
                c = fc, d = (long long)fd * k + s % k;
            }
            xc ^= c, xd ^= d;
        }
        puts(xc || xd ? "Win" : "Loss");
    }
}
