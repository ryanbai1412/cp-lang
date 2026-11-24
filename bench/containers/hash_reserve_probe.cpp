// Matched C++ for hash_reserve_probe.cp: a flat linear-probing table of (key, count) with a used
// byte per slot, sized once for n at a load of at most 3/4, the same salted splitmix64 hash.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
static uint64_t mix(uint64_t x) {
    x += 0x9e3779b97f4a7c15ULL;
    x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
    x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
    return x ^ (x >> 31);
}
int main() {
    const ll n = 3375000;
    const size_t cap = n + (n + 2) / 3;
    const uint64_t salt = chrono::steady_clock::now().time_since_epoch().count();
    vector<pair<ll, ll>> e(cap);
    vector<uint8_t> used(cap);
    auto find = [&](ll k) {
        size_t i = (size_t)(((unsigned __int128)(mix(k ^ salt) * 0x9e3779b97f4a7c15ULL) * cap) >> 64);
        while (used[i] && e[i].first != k) i = i + 1 == cap ? 0 : i + 1;
        return i;
    };
    ll x = 987654321, size = 0;
    for (ll i = 0; i < n; ++i) {
        x = (x * 1103515245 + 12345) % 2147483647;
        size_t j = find(x - 1000000000);
        if (!used[j]) used[j] = 1, e[j] = {x - 1000000000, 0}, ++size;
        ++e[j].second;
    }
    ll hits = 0;
    x = 987654321;
    for (ll i = 0; i < n; ++i) {
        x = (x * 1103515245 + 12345) % 2147483647;
        size_t j = find(i % 2 == 0 ? x - 1000000000 : x + 2000000000);
        if (used[j]) hits += e[j].second;
    }
    printf("%lld %lld\n", size, hits);
}
