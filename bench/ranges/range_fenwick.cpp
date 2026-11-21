#include <bits/stdc++.h>
using namespace std;

int main() {
    const int n = 200000;
    vector<long long> b1(n + 1, 0), b2(n + 1, 0);
    auto add_at = [&](int p, long long v1, long long v2) {
        for (; p <= n; p += p & -p) {
            b1[p] += v1;
            b2[p] += v2;
        }
    };
    auto pref = [&](int k) {
        long long s1 = 0, s2 = 0;
        for (int p = k; p > 0; p -= p & -p) {
            s1 += b1[p];
            s2 += b2[p];
        }
        return s1 * k - s2;
    };
    long long s = 0, x = 1234567;
    for (int i = 0; i < 200000; i++) {
        x = (x * 1103515245LL + 12345) & 2147483647LL;
        int l = x % n;
        x = (x * 1103515245LL + 12345) & 2147483647LL;
        int r = x % n;
        int a = min(l, r), b = max(l, r);
        long long v = (x % 17) - 8;
        add_at(a + 1, v, v * a);
        if (b + 1 < n) add_at(b + 2, -v, -v * (b + 1));
        s += pref(b + 1) - pref(a);
    }
    cout << s << "\n";
}
