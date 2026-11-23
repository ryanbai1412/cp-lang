#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int t; cin >> t;
    while (t--) {
        int n; cin >> n;
        vector<long long> a(n);
        for (auto& x : a) cin >> x;
        vector<long long> d(n - 1);
        long long G = 0;
        for (int i = 0; i + 1 < n; i++) d[i] = llabs(a[i] - a[i + 1]), G = max(G, d[i]);
        // gaps 0-based: d[i] between a[i], a[i+1]; segment [l, r] 0-based inclusive
        // replaces gap l-1 by |a[l-1] - 2a[r]|, gap r by |2a[l] - a[r+1]|, doubles gaps l..r-1.
        auto ok = [&](long long X) {
            vector<int> bad;
            for (int i = 0; i + 1 < n; i++) if (d[i] > X) { bad.push_back(i); if (bad.size() > 2) return false; }
            if (bad.empty()) return true;
            auto fits = [&](int l, int r) {
                if (l > 0 && llabs(a[l - 1] - 2 * a[r]) > X) return false;
                if (r + 1 < n && llabs(2 * a[l] - a[r + 1]) > X) return false;
                return true;
            };
            if (bad.size() == 2) {
                int l = bad[0] + 1, r = bad[1];
                for (int i = l; i < r; i++) if (2 * d[i] > X) return false;
                return fits(l, r);
            }
            int b = bad[0];
            for (int l = b + 1, r = l; r < n; r++) {  // gap b is the left boundary
                if (r > l && 2 * d[r - 1] > X) break;
                if (fits(l, r)) return true;
            }
            for (int r = b, l = r; l >= 0; l--) {  // gap b is the right boundary
                if (l < r && 2 * d[l] > X) break;
                if (fits(l, r)) return true;
            }
            return false;
        };
        long long lo = 0, hi = G;
        while (lo < hi) { long long m = (lo + hi) / 2; if (ok(m)) hi = m; else lo = m + 1; }
        cout << lo << '\n';
    }
}
