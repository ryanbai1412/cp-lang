#include <bits/stdc++.h>
using namespace std;
using ll = long long;

int main() {
    const int n = 1000000;
    map<int, int> mp;
    mp[0] = 0;
    auto split = [&](int x) {
        if (x >= n) return mp.end();
        auto it = mp.lower_bound(x);
        if (it != mp.end() && it->first == x) return it;
        --it;
        return mp.emplace_hint(next(it), x, it->second);
    };
    uint64_t s = 987654321ULL;
    ll total = 0;
    for (int q = 0; q < 200000; ++q) {
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        int a = (int)((s >> 33) % n);
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        int len = (int)((s >> 33) % 5000) + 1;
        int r = min(n, a + len);
        int val = q % 17 + 1;
        auto it_r = split(r);
        auto it_l = split(a);
        for (auto it = it_l; it != it_r; ++it) {
            auto nxt = next(it);
            int seg_r = (nxt == mp.end()) ? n : nxt->first;
            total += (ll)(seg_r - it->first) * it->second;
        }
        mp.erase(it_l, it_r);
        auto ins = mp.emplace(a, val).first;
        if (ins != mp.begin() && prev(ins)->second == val) {
            mp.erase(ins);
            ins = prev(mp.lower_bound(a));
        }
        auto nxt = next(ins);
        if (nxt != mp.end() && nxt->second == val) mp.erase(nxt);
    }
    cout << total << "\n";
}
