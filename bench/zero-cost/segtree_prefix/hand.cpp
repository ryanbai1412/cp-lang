#include <bits/stdc++.h>
using namespace std;

int64_t kernel(int n, const vector<int64_t>& a, const vector<tuple<int, int, int64_t>>& qs) {
    vector<int64_t> p(n + 1, 0);
    for (int i = 0; i < n; ++i) p[i + 1] = p[i] + a[i];
    int size = 1;
    while (size < n) size *= 2;
    vector<int64_t> t(2 * size, 0);
    for (int i = 0; i < n; ++i) t[size + i] = a[i];
    for (int k = size - 1; k >= 1; --k) t[k] = t[2 * k] + t[2 * k + 1];
    int64_t acc = 0;
    for (auto [kind, x, y] : qs) {
        if (kind == 1) {
            int idx = x + size;
            t[idx] = y;
            for (idx >>= 1; idx >= 1; idx >>= 1) {
                t[idx] = t[2 * idx] + t[2 * idx + 1];
            }
        } else {
            int l = x + size;
            int r = (int)y + 1 + size;
            int64_t s = 0;
            while (l < r) {
                if (l & 1) s += t[l++];
                if (r & 1) s += t[--r];
                l >>= 1;
                r >>= 1;
            }
            acc = (acc + s + (p[(int)y + 1] - p[x])) ^ (acc >> 4);
        }
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
    vector<tuple<int, int, int64_t>> qs(q);
    for (int i = 0; i < q; ++i) {
        int kind, x;
        int64_t y;
        cin >> kind >> x >> y;
        qs[i] = {kind, x, y};
    }
    cout << kernel(n, a, qs) << '\n';
}
