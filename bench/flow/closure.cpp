#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
#include "_dinic.inc"
const int N = 50000;
int main() {
    vector<long long> profit(N); for (int v = 0; v < N; v++) profit[v] = nxt(2000001) - 1000000;
    vector<pair<int, int>> needs; for (int i = 0; i < 300000; i++) { int u = nxt(N), v = nxt(N); needs.push_back({u, v}); }
    int s = N, t = N + 1; Dinic g(N + 2); long long gain = 0;
    for (int v = 0; v < N; v++) { if (profit[v] > 0) { g.add(s, v, profit[v]); gain += profit[v]; } else if (profit[v] < 0) g.add(v, t, -profit[v]); }
    for (auto [u, v] : needs) g.add(u, v, 1LL << 60);
    long long value = gain - g.max_flow(s, t);
    auto side = g.min_cut(s); int taken = 0; for (int v = 0; v < N; v++) taken += side[v];
    printf("%lld %d\n", value, taken);
}
