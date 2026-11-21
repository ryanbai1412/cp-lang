#include "common.h"
int main() {
    int n = rd(), k = rd();
    vector<ll> a(k), b(k), c(k);
    for (int i = 0; i < k; i++) a[i] = rd(), b[i] = rd(), c[i] = rd();
    vector<ll> dist(n, LLONG_MAX);
    vector<int> order{0};
    vector<size_t> at(k);
    dist[0] = 0;
    while (true) {
        int best = -1; ll bd = 0; int bt = 0;
        for (int i = 0; i < k; i++)
            for (; at[i] < order.size(); at[i]++) {
                int s = order[at[i]], t = (a[i] * s + b[i]) % n;
                if (dist[t] != LLONG_MAX) continue;
                ll d = dist[s] + c[i];
                if (best < 0 || d < bd) best = i, bd = d, bt = t;
                break;
            }
        if (best < 0) break;
        dist[bt] = bd; order.push_back(bt); at[best]++;
    }
    ll total = 0; for (ll d : dist) if (d != LLONG_MAX) total += d;
    printf("%lld\n", total);
}
