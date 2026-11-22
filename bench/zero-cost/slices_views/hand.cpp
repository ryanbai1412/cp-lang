#include <bits/stdc++.h>
using namespace std;

int64_t kernel(int n, const vector<int64_t>& a, const vector<pair<int, int>>& qs) {
    vector<int64_t> arr(a.begin(), a.end());
    int64_t acc = 0;
    for (auto [l, r] : qs) {
        int64_t s = 0;
        for (int i = l; i < r; ++i) {
            s = (s + arr[i] * 5) ^ (s >> 4);
        }
        acc ^= s;
    }
    return acc;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, q;
    if (!(cin >> n >> q)) return 0;
    vector<int64_t> a(n);
    for (int i = 0; i < n; ++i) cin >> a[i];
    vector<pair<int, int>> qs(q);
    for (int i = 0; i < q; ++i) cin >> qs[i].first >> qs[i].second;
    cout << kernel(n, a, qs) << '\n';
}
