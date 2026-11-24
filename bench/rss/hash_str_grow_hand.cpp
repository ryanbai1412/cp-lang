// Matched C++ for hash_str_grow.cp: a linear-probing table of (string, value) with a used byte per
// slot, grown by 1.5x past a load of 3/4 by moving every entry into a new table, then freeing the old.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
struct Table {
    vector<pair<string, ll>> e = vector<pair<string, ll>>(8);
    vector<uint8_t> used = vector<uint8_t>(8);
    size_t n = 0;
    size_t find(const string& k) const {
        size_t cap = e.size(), i = hash<string>()(k) % cap;
        while (used[i] && e[i].first != k) i = i + 1 == cap ? 0 : i + 1;
        return i;
    }
    void grow() {
        Table t;
        t.e.assign(e.size() + e.size() / 2, {});
        t.used.assign(t.e.size(), 0);
        for (size_t i = 0; i < e.size(); ++i)
            if (used[i]) { size_t j = t.find(e[i].first); t.used[j] = 1; t.e[j] = std::move(e[i]); }
        t.n = n;
        *this = std::move(t);
    }
    ll& operator[](const string& k) {
        size_t i = find(k);
        if (used[i]) return e[i].second;
        if ((n + 1) * 4 > e.size() * 3) { grow(); i = find(k); }
        used[i] = 1; e[i] = {k, 0}; ++n;
        return e[i].second;
    }
};
int main() {
    ll n;
    if (scanf("%lld", &n) != 1) return 1;
    Table m;
    ll x = 987654321;
    for (ll i = 0; i < n; i++) {
        x = (x * 1103515245LL + 12345LL) % 2147483647LL;
        m["k" + to_string(x)] = i;
    }
    ll sum = 0;
    x = 987654321;
    for (ll i = 0; i < n; i++) {
        x = (x * 1103515245LL + 12345LL) % 2147483647LL;
        size_t j = m.find((i % 2 == 0 ? "k" : "m") + to_string(x));
        if (m.used[j]) sum += m.e[j].second;
    }
    printf("%lld %lld\n", (ll)m.n, sum);
}
