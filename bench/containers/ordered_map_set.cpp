#include <bits/stdc++.h>
using namespace std;
using ll = long long;

int main() {
    map<ll, ll> m;
    set<ll> s;
    uint64_t rng = 987654321ULL;
    for (int i = 0; i < 100000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        ll k = (ll)((rng >> 33) % 10000000ULL);
        m[k] = i;
        s.insert(k);
    }
    ll acc = 0;
    for (int i = 0; i < 100000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        ll q = (ll)((rng >> 33) % 10000000ULL);
        auto it = m.lower_bound(q);
        if (it != m.end()) acc += it->first + it->second;
        auto sit = s.lower_bound(q);
        if (sit != s.end()) acc += *sit;
    }
    for (int i = 0; i < 100000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        ll lo = (ll)((rng >> 33) % 9900000ULL);
        ll hi = lo + 50000;
        auto it = m.lower_bound(lo);
        if (it != m.end() && it->first < hi) acc += it->first + it->second;
        auto sit = s.lower_bound(hi);
        if (sit != s.begin()) {
            --sit;
            if (*sit >= lo) acc += *sit;
        }
    }
    for (int i = 0; i < 20000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        ll lo = (ll)((rng >> 33) % 9999000ULL);
        ll hi = lo + 500;
        for (auto it = m.lower_bound(lo); it != m.end() && it->first < hi; ++it) {
            acc += it->first + it->second;
        }
        for (auto it = s.lower_bound(lo); it != s.end() && *it < hi; ++it) {
            acc += *it;
        }
    }
    cout << acc + (ll)m.size() + (ll)s.size() << "\n";
}
