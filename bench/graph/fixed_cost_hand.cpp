#include "common.h"
int main() {
    int n = rd(), k = rd();
    vector<ll> a(k), b(k), c(k);
    for (int i = 0; i < k; i++) a[i] = rd(), b[i] = rd(), c[i] = rd();
    vector<ll> dist(n, LLONG_MAX);
    vector<char> done(n);
    vector<vector<pair<ll, int>>> q(k + 1);
    vector<size_t> head(k + 1);
    dist[0] = 0; q[0].push_back({0, 0});
    while (true) {
        int best = -1;
        for (int i = 0; i <= k; i++)
            if (head[i] < q[i].size() && (best < 0 || q[i][head[i]].first < q[best][head[best]].first)) best = i;
        if (best < 0) break;
        auto [d, s] = q[best][head[best]++];
        if (done[s]) continue; done[s] = 1;
        for (int i = 0; i < k; i++) {
            int t = (a[i] * s + b[i]) % n; ll nd = d + c[i];
            if (nd < dist[t]) dist[t] = nd, q[i + 1].push_back({nd, t});
        }
    }
    ll total = 0; for (ll d : dist) if (d != LLONG_MAX) total += d;
    printf("%lld\n", total);
}
