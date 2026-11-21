#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
#include "_dinic.inc"
const int N = 5000;
int main() {
    int s = 0, t = N - 1;
    struct A { int u, v; long long lo, hi; }; vector<A> arcs;
    for (int i = 0; i < 200000; i++) { int u = nxt(N), v = nxt(N); long long hi = nxt(1000) + 1; long long lo = nxt(hi / 4 + 1); arcs.push_back({u, v, lo, hi}); }
    for (int v = 0; v < N; v++) { arcs.push_back({s, v, 0, 1000000}); arcs.push_back({v, t, 0, 1000000}); }
    int ss = N, tt = N + 1; Dinic g(N + 2); vector<long long> excess(N);
    vector<int> ids; for (auto& a : arcs) ids.push_back(g.add(a.u, a.v, a.hi - a.lo));
    for (auto& a : arcs) { excess[a.v] += a.lo; excess[a.u] -= a.lo; }
    long long demand = 0;
    for (int v = 0; v < N; v++) { if (excess[v] > 0) { g.add(ss, v, excess[v]); demand += excess[v]; } else if (excess[v] < 0) g.add(v, tt, -excess[v]); }
    int back = g.add(t, s, 1LL << 60);
    if (g.max_flow(ss, tt) != demand) { puts("none"); return 0; }
    long long base = g.flow(back); g.close(back);
    long long value = base + g.max_flow(s, t);
    long long sum = 0; for (size_t e = 0; e < arcs.size(); e++) sum += arcs[e].lo + g.flow(ids[e]);
    printf("%lld %lld\n", value, sum);
}
