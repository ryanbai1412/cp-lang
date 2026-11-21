#include <bits/stdc++.h>
#include <ext/pb_ds/assoc_container.hpp>
#include <ext/pb_ds/tree_policy.hpp>
using namespace std;
using namespace __gnu_pbds;
using ll = long long;
using oset = tree<ll, null_type, less<ll>, rb_tree_tag, tree_order_statistics_node_update>;

int main() {
    oset s;
    uint64_t rng = 246813579ULL;
    vector<ll> keys;
    keys.reserve(100000);
    for (int i = 0; i < 100000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        ll x = (ll)((rng >> 33) % 10000000ULL);
        s.insert(x);
        keys.push_back(x);
    }
    ll acc = 0;
    for (int i = 0; i < 50000; ++i) {
        if (s.erase(keys[i * 2])) acc += 1;
    }
    size_t sz = s.size();
    for (int i = 0; i < 100000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        ll q = (ll)((rng >> 33) % 10000000ULL);
        acc += (ll)s.order_of_key(q);
    }
    for (int i = 0; i < 100000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        size_t idx = (size_t)((rng >> 33) % sz);
        acc += *s.find_by_order(idx);
    }
    cout << acc + (ll)s.size() << "\n";
}
