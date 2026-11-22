#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    long long x;
    cin >> n >> x;
    vector<long long> w(n);
    for (auto& v : w) cin >> v;
    vector<pair<int, long long>> d(1 << n, {99, 0});
    d[0] = {1, 0};
    for (int m = 0; m < (1 << n); m++)
        for (int p = 0; p < n; p++)
            if (!(m >> p & 1)) {
                auto c = d[m];
                if (c.second + w[p] <= x) c.second += w[p];
                else c.first++, c.second = w[p];
                d[m | 1 << p] = min(d[m | 1 << p], c);
            }
    cout << d[(1 << n) - 1].first << '\n';
}
