#include <bits/stdc++.h>
using namespace std;

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    long long n, q;
    if (!(cin >> n >> q)) return 0;
    vector<long long> a(n);
    for (auto& x : a) cin >> x;
    long long total = 0;
    for (long long k = 0; k < q; ++k) {
        long long l = k * 7919 % (n - 8), hi = 0, lo = 0;
        for (long long i = l; i < l + 8; ++i) {
            long long x = a[i], h = max(hi, x), second = 0;
            for (long long y : {hi, lo, x, 0LL})
                if (y < h) second = max(second, y);
            hi = h, lo = second;
        }
        total += hi + lo;
    }
    cout << total << '\n';
}
