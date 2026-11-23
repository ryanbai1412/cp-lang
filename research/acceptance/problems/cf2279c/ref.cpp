// Final values b <= a, wlog b >= 0 (0 beats any negative). With b sorted descending only
// b1 * b2 < l matters, so the largest a keeps t1 and every other is capped at t2 <= t1:
// enumerate t2 in [0, min(a2, isqrt(l - 1))], t1 = (l - 1) / t2, a pointer tracks the cap sum.
#include <bits/stdc++.h>
using namespace std;
int main() {
    int t;
    scanf("%d", &t);
    while (t--) {
        int n;
        long long l;
        scanf("%d %lld", &n, &l);
        vector<long long> a(n);
        for (auto &x : a) scanf("%lld", &x);
        sort(a.rbegin(), a.rend());
        long long L = l - 1, total = 0;
        for (int i = 1; i < n; i++) total += a[i];
        long long best = total;  // t2 = 0
        long long r = 0;
        while ((r + 1) * (r + 1) <= L) r++;
        long long hi = min(a[1], r);
        int k = n;  // a[1..k) are > t2
        long long sumBig = total;
        for (long long t2 = 1; t2 <= hi; t2++) {
            while (k > 1 && a[k - 1] <= t2) sumBig -= a[--k];
            long long cost = sumBig - (long long)(k - 1) * t2 + max(0LL, a[0] - L / t2);
            best = min(best, cost);
        }
        printf("%lld\n", best);
    }
}
