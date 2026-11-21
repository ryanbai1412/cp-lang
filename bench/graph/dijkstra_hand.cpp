#include "common.h"
int main() {
    int n = rd(), m = rd();
    vector<int> u(m), v(m); vector<ll> w(m);
    for (int i = 0; i < m; i++) u[i] = rd() - 1, v[i] = rd() - 1, w[i] = rd();
    Csr g(n, u, v);
    vector<ll> dist(n, LLONG_MAX);
    priority_queue<pair<ll, int>, vector<pair<ll, int>>, greater<>> pq;
    dist[0] = 0; pq.push({0, 0});
    while (!pq.empty()) {
        auto [d, x] = pq.top(); pq.pop();
        if (d > dist[x]) continue;
        for (int i = g.start[x]; i < g.start[x + 1]; i++) {
            int y = g.to[i]; ll nd = d + w[g.e[i]];
            if (nd < dist[y]) dist[y] = nd, pq.push({nd, y});
        }
    }
    ll total = 0; for (ll d : dist) if (d != LLONG_MAX) total += d;
    printf("%lld\n", total);
}
