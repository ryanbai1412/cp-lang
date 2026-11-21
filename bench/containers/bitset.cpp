#include <bits/stdc++.h>
using namespace std;
using ll = long long;

int main() {
    constexpr int M = 200001;
    vector<int> weights;
    weights.reserve(4000);
    uint64_t s = 123456789ULL;
    for (int i = 0; i < 4000; ++i) {
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        weights.push_back((int)((s >> 33) % 100) + 1);
    }
    vector<int> cnt(M, 0);
    for (int w : weights) if (w > 0 && w < M) cnt[w]++;
    bitset<M> reach;
    reach.set(0);
    for (int w = 1; w < M; ++w) {
        int c = cnt[w];
        if (!c) continue;
        int k = 1;
        while (c > 0) {
            int take = min(k, c);
            int sh = take * w;
            if (sh < M) reach |= (reach << sh);
            c -= take;
            k <<= 1;
        }
    }
    bitset<M> a = reach;
    bitset<M> b = reach << 37;
    bitset<M> c = reach >> 19;
    ll ans = (ll)reach.count();
    for (int step = 0; step < 200; ++step) {
        a |= (a << (step % 31 + 1));
        ans += (ll)(a & b & c).count();
    }
    cout << ans << "\n";
}
