#include <bits/stdc++.h>
#include <ext/pb_ds/assoc_container.hpp>
using namespace std;
using ll = long long;

struct SplitMix {
    static uint64_t splitmix64(uint64_t x) {
        x += 0x9e3779b97f4a7c15ULL;
        x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
        x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
        return x ^ (x >> 31);
    }
    size_t operator()(ll x) const {
        static const uint64_t r = chrono::steady_clock::now().time_since_epoch().count();
        return splitmix64((uint64_t)x + r);
    }
};

int main() {
    __gnu_pbds::gp_hash_table<ll, ll, SplitMix> mp;
    uint64_t s = 314159265ULL;
    for (int i = 0; i < 300000; ++i) {
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        ll k = (ll)((s >> 33) % 200000);
        mp[k] = i;
    }
    ll ans = 0;
    for (int i = 0; i < 1000000; ++i) {
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        ll k = (ll)((s >> 33) % 400000);
        auto it = mp.find(k);
        if (it != mp.end()) ans += it->second;
    }
    cout << ans << "\n";
}
