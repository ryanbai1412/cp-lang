#include "common.h"
int main() {
    int n = rd(), m = rd();
    vector<int> u(m), v(m);
    for (int i = 0; i < m; i++) u[i] = rd() - 1, v[i] = rd() - 1;
    Csr g(n, u, v);
    vector<int> dist(n, -1), q; q.reserve(n);
    dist[0] = 0; q.push_back(0);
    for (size_t h = 0; h < q.size(); h++) {
        int x = q[h];
        for (int i = g.start[x]; i < g.start[x + 1]; i++)
            if (dist[g.to[i]] < 0) dist[g.to[i]] = dist[x] + 1, q.push_back(g.to[i]);
    }
    ll total = 0; for (int d : dist) if (d >= 0) total += d;
    printf("%lld\n", total);
}
