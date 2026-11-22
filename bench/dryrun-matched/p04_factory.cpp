#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    long long n, t;
    cin >> n >> t;
    vector<long long> k(n);
    for (auto& x : k) cin >> x;
    long long lo = 1, hi = 1e18;
    while (lo < hi) {
        long long m = lo + (hi - lo) / 2, s = 0;
        for (auto x : k) {
            s += m / x;
            if (s >= t) break;
        }
        if (s >= t) hi = m;
        else lo = m + 1;
    }
    cout << lo << '\n';
}
