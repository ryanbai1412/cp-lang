// f(n): runs of Operation 1 of distinct lengths a >= 2 form a mask V (bits 0, 1 clear);
// f(n) = min over V with V - 2 popcount(V) >= n - 1 of V + sum(a - 1) - n, and f(0) = 0, f(1) = 1.
// For each p, the best V >= T = n - 1 + 2p with popcount <= p is T itself or a prefix of T
// plus one bit j that T lacks, lower bits cleared. O(log^2) per n.
#include <bits/stdc++.h>
using namespace std;
typedef unsigned long long u64;
static long long cost(u64 v) {
    long long c = (long long)v;
    for (u64 w = v; w; w &= w - 1) c += __builtin_ctzll(w) - 1;
    return c;
}
static long long f(u64 n) {
    if (n <= 1) return n;
    u64 N = n - 1;
    long long best = LLONG_MAX;  // V + sum(a - 1) >= V, and V grows with p and j
    for (int p = 1; p <= 61; p++) {
        u64 T = N + 2 * p;
        if (T >= (u64)best) break;
        if ((T & 3) == 0 && __builtin_popcountll(T) <= p) best = min(best, cost(T));
        for (int j = 2; j <= 61; j++) {
            if (T >> j & 1) continue;
            u64 v = ((T >> (j + 1)) << (j + 1)) | (1ULL << j);
            if (v >= (u64)best) break;
            if (__builtin_popcountll(v) <= p) best = min(best, cost(v));
        }
    }
    return best - (long long)n;
}
int main() {
    int t;
    if (scanf("%d", &t) != 1) return 0;
    while (t--) {
        u64 l, r;
        if (scanf("%llu %llu", &l, &r) != 2) return 0;
        long long s = 0;
        for (u64 n = l; n <= r; n++) s = (s + f(n)) % 998244353;
        printf("%lld\n", s);
    }
}
