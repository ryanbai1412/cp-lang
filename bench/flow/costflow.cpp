#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
const int N = 2002;
vector<int> head[N]; vector<int> to; vector<long long> lft, price;
void add(int u, int v, long long c, long long w) {
    head[u].push_back(to.size()); to.push_back(v); lft.push_back(c); price.push_back(w);
    head[v].push_back(to.size()); to.push_back(u); lft.push_back(0); price.push_back(-w);
}
int main() {
    int s = 2000, t = 2001;
    for (int i = 0; i < 1000; i++) { add(s, i, 5, 0); add(1000 + i, t, 5, 0); }
    for (int k = 0; k < 60000; k++) { int u = nxt(1000); int v = nxt(1000); long long c = nxt(5) + 1; add(u, 1000 + v, c, nxt(10000)); }
    vector<long long> pot(N, 0); long long flow = 0, cost = 0;
    while (true) {
        vector<long long> dist(N, LLONG_MAX); vector<int> via(N, -1); vector<char> done(N, 0);
        dist[s] = 0; priority_queue<pair<long long, int>, vector<pair<long long, int>>, greater<>> pq; pq.push({0, s});
        while (!pq.empty()) { auto [d, u] = pq.top(); pq.pop(); if (done[u]) continue; done[u] = 1; if (u == t) break;
            for (int a : head[u]) { if (!lft[a]) continue; int v = to[a]; long long nd = d + price[a] + pot[u] - pot[v];
                if (nd < dist[v]) { dist[v] = nd; via[v] = a; pq.push({nd, v}); } } }
        if (!done[t]) break;
        for (int v = 0; v < N; v++) pot[v] += done[v] ? dist[v] : dist[t];
        long long amt = LLONG_MAX; for (int v = t; v != s; v = to[via[v] ^ 1]) amt = min(amt, lft[via[v]]);
        for (int v = t; v != s; v = to[via[v] ^ 1]) { lft[via[v]] -= amt; lft[via[v] ^ 1] += amt; }
        flow += amt; cost += amt * (pot[t] - pot[s]);
    }
    printf("%lld %lld\n", flow, cost);
}
