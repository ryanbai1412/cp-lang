// For each l, greedily match "202" ending at p; s[l..r] is bad iff next5(p) <= r < next6(p).
// Each l adds a constant then a decreasing ramp to the bad counts: two difference arrays. O(n).
#include <bits/stdc++.h>
using namespace std;
int main() {
    int t;
    scanf("%d", &t);
    static char buf[300005];
    while (t--) {
        int n;
        scanf("%d %s", &n, buf);
        vector<array<int, 4>> nx(n + 2);  // next index >= i of '0','2','5','6'; n = none
        const char ds[4] = {'0', '2', '5', '6'};
        nx[n] = nx[n + 1] = {n, n, n, n};
        for (int i = n - 1; i >= 0; i--) {
            nx[i] = nx[i + 1];
            for (int c = 0; c < 4; c++)
                if (buf[i] == ds[c]) nx[i][c] = i;
        }
        vector<long long> C(n + 1, 0), K(n + 1, 0);
        for (int l = 0; l < n; l++) {
            int p = nx[l][1];
            if (p < n) p = nx[p + 1][0];
            if (p < n) p = nx[p + 1][1];
            if (p >= n) continue;
            int a = nx[p + 1][2], b = nx[p + 1][3] - 1;
            if (a > b) continue;
            C[l] += b - a + 1, C[a + 1] -= b - a + 1;
            C[a + 1] += b + 1, C[b + 1] -= b + 1;
            K[a + 1] -= 1, K[b + 1] += 1;
        }
        long long c = 0, k = 0;
        string out;
        for (int i = 0; i < n; i++) {
            c += C[i], k += K[i];
            out += to_string((long long)(i + 1) * (n - i) - (c + k * i));
            out += i + 1 < n ? ' ' : '\n';
        }
        fputs(out.c_str(), stdout);
    }
}
