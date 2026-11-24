// Matched C++ for hash_array_param.cp: four flat linear-probing tables of 32-bit (key, value) with a
// used byte per slot, each sized once for n at a load of at most 3/4, the same salted splitmix64 hash.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
static uint64_t mix(uint64_t x) {
    x += 0x9e3779b97f4a7c15ULL;
    x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
    x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
    return x ^ (x >> 31);
}
struct Table {
    size_t cap = 0;
    uint64_t salt = 0;
    vector<pair<int32_t, uint32_t>> e;
    vector<uint8_t> used;
    size_t find(int32_t k) const {
        size_t i = (size_t)(((unsigned __int128)(mix(k ^ salt) * 0x9e3779b97f4a7c15ULL) * cap) >> 64);
        while (used[i] && e[i].first != k) i = i + 1 == cap ? 0 : i + 1;
        return i;
    }
};
static void fill(Table& t, ll n, ll seed) {
    t.cap = n + (n + 2) / 3;
    t.salt = chrono::steady_clock::now().time_since_epoch().count();
    t.e.assign(t.cap, {});
    t.used.assign(t.cap, 0);
    ll x = seed;
    for (ll i = 0; i < n; ++i) {
        x = (x * 1103515245 + 12345) % 2147483647;
        int32_t k = (int32_t)(x % 1000000001);
        size_t j = t.find(k);
        t.used[j] = 1, t.e[j] = {k, (uint32_t)(i + 1)};
    }
}
int main() {
    ll n;
    if (scanf("%lld", &n) != 1) return 1;
    vector<Table> maps(4);
    for (int r = 0; r < 4; ++r) fill(maps[r], n, 987654321 + r);
    ll size = 0, hits = 0;
    for (auto& t : maps) size += count(t.used.begin(), t.used.end(), 1);
    for (int r = 0; r < 4; ++r) {
        ll x = 987654321 + r;
        for (ll i = 0; i < n; ++i) {
            x = (x * 1103515245 + 12345) % 2147483647;
            hits += maps[r].e[maps[r].find((int32_t)(x % 1000000001))].second;
        }
    }
    printf("%lld %lld\n", size, hits);
}
