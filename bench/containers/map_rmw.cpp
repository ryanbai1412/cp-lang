// Matched C++ for map_rmw.cp: std::map with a (ll, bool) key, one `m[k] += v` walk per update.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
struct Key {
    ll a;
    bool b;
    bool operator<(const Key& o) const { return tie(a, b) < tie(o.a, o.b); }
};
int main() {
    map<Key, ll> m;
    uint64_t rng = 987654321;
    for (ll i = 0; i < 4000000; i++) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        ll r = (ll)((rng >> 33) % 100000);
        m[Key{r / 2, r % 2 == 1}] += i % 1000;
    }
    ll acc = 0;
    for (auto& [k, v] : m) acc = (acc * 31 + k.a + v) % 1000000007;
    printf("%lld\n", acc + (ll)m.size());
}
