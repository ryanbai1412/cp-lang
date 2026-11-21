#include "common.h"
int main() {
    int n = rd(), m = rd();
    vector<int> u(m), v(m), w(m);
    for (int i = 0; i < m; i++) u[i] = rd() - 1, v[i] = rd() - 1, w[i] = rd();
    Csr g(n, u, v);
    vector<int> dist(n, INT_MAX);
    deque<int> q; dist[0] = 0; q.push_back(0);
    vector<char> done(n);
    while (!q.empty()) {
        int x = q.front(); q.pop_front();
        if (done[x]) continue; done[x] = 1;
        for (int i = g.start[x]; i < g.start[x + 1]; i++) {
            int y = g.to[i], c = w[g.e[i]];
            if (dist[x] + c < dist[y]) { dist[y] = dist[x] + c; if (c) q.push_back(y); else q.push_front(y); }
        }
    }
    ll total = 0; for (int d : dist) if (d != INT_MAX) total += d;
    printf("%lld\n", total);
}
