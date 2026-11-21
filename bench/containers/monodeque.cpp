#include <bits/stdc++.h>
using namespace std;
using ll = long long;

int main() {
    const int n = 2000000, w = 1000;
    vector<pair<int, ll>> q;
    q.reserve(n);
    size_t head = 0;
    uint64_t s = 246813579ULL;
    ll ans = 0;
    for (int i = 0; i < n; ++i) {
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        ll v = (ll)((s >> 33) % 1000000);
        while (q.size() > head && q.back().second >= v) q.pop_back();
        q.push_back({i, v});
        while (head < q.size() && q[head].first < i - w + 1) ++head;
        ans += q[head].second;
    }
    cout << ans << "\n";
}
