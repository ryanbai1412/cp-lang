// Not a judge-accepted submission: written for the corpus and checked against the sample and an exhaustive brute.
// Sell for the longest asked span P backwards in time (a unit is sellable on day d only if it
// spoils on day d or later): each day take the m most valuable units on hand, a type's first unit
// worth a + s. A shorter span keeps the best m p of those units, so drop the cheapest ones in turn.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
int main() {
    int n, m, k;
    scanf("%d %d %d", &n, &m, &k);
    vector<ll> a(n), s(n), c(n), x(n), sold(n, 0);
    for (int i = 0; i < n; i++) scanf("%lld %lld %lld %lld", &a[i], &s[i], &c[i], &x[i]);
    vector<int> q(k);
    for (auto& v : q) scanf("%d", &v);
    int P = *max_element(q.begin(), q.end());
    vector<vector<int>> starts(P + 1);
    for (int i = 0; i < n; i++) {
        ll last = x[i] ? (c[i] + x[i] - 1) / x[i] : P;
        starts[min<ll>(last, P)].push_back(i);
    }
    auto avail = [&](int i, int d) { return x[i] ? c[i] - (ll)(d - 1) * x[i] : c[i]; };  // units lasting to day d
    auto worth = [&](int i) { return sold[i] ? a[i] : a[i] + s[i]; };
    priority_queue<pair<ll, int>> pq;
    vector<pair<ll, ll>> units;  // (value, count)
    vector<int> held;
    for (int d = P; d >= 1; d--) {
        for (int i : starts[d]) pq.push({worth(i), i});
        for (int i : held) if (avail(i, d) > sold[i]) pq.push({worth(i), i});
        held.clear();
        ll room = m;
        while (room && !pq.empty()) {
            auto [w, i] = pq.top();
            pq.pop();
            ll take = sold[i] ? min(room, avail(i, d) - sold[i]) : 1;
            units.push_back({w, take});
            sold[i] += take, room -= take;
            if (avail(i, d) > sold[i]) pq.push({worth(i), i});
            else held.push_back(i);
        }
    }
    sort(units.begin(), units.end());
    ll cnt = 0, sum = 0;
    for (auto [w, t] : units) cnt += t, sum += w * t;
    vector<ll> best(P + 1);
    size_t lo = 0;
    for (int p = P; p >= 1; p--) {
        while (cnt > (ll)m * p) {
            ll drop = min(units[lo].second, cnt - (ll)m * p);
            sum -= drop * units[lo].first, cnt -= drop, units[lo].second -= drop;
            if (!units[lo].second) lo++;
        }
        best[p] = sum;
    }
    for (int v : q) printf("%lld\n", v ? best[v] : 0LL);
}
