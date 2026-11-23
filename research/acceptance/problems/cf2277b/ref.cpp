#include <bits/stdc++.h>
using namespace std;
int main() {
    int t;
    if (scanf("%d", &t) != 1) return 0;
    while (t--) {
        int n;
        if (scanf("%d", &n) != 1) return 0;
        long long mn = LLONG_MAX;
        for (int i = 0; i < n; i++) {
            long long x;
            if (scanf("%lld", &x) != 1) return 0;
            mn = min(mn, x);
        }
        printf("%d\n", mn <= n - 1 ? n : 0);
    }
}
