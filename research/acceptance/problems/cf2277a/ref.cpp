#include <bits/stdc++.h>
using namespace std;
int main() {
    int t;
    scanf("%d", &t);
    while (t--) {
        int n, k;
        scanf("%d %d", &n, &k);
        vector<int> cnt(n + 2, 0);
        for (int i = 0; i < n; i++) {
            int x;
            scanf("%d", &x);
            cnt[x]++;
        }
        int m = 0;
        long long pre = 0;
        while (m <= n) {
            pre += cnt[m];
            if (cnt[m] == 0 || pre < (long long)k * (m + 1)) break;
            m++;
        }
        printf("%d\n", m);
    }
}
