// Official editorial construction. Impossible when N = K, N is odd, or v2(N) = 1 < v2(K) + 1.
// K odd: alternate 1, -1. Otherwise (v2(N) >= 2) a -1 flips the K window products ending at
// it; with K <= N/2 (the same array serves N - K), flip disjoint runs of K windows and pull
// the last one back so exactly N/2 products are negative.
#include <bits/stdc++.h>
using namespace std;
int main() {
    long long n, k;
    scanf("%lld %lld", &n, &k);
    auto v2 = [](long long x) { return __builtin_ctzll(x); };
    if (n == k || n % 2 || (v2(n) == 1 && k % 2 == 0)) {
        puts("0");
        return 0;
    }
    vector<int> a(n, 1);
    if (k % 2) {
        for (long long i = 1; i < n; i += 2) a[i] = -1;
    } else {
        if (k > n / 2) k = n - k;
        long long half = n / 2, m = (half + k - 1) / k, o = (m * k - half) / 2;
        for (long long j = 1; j <= m; j++) a[j * k - 1 - (j == m ? o : 0)] = -1;
    }
    string out;
    for (long long i = 0; i < n; i++) out += (i ? " " : ""), out += a[i] < 0 ? "-1" : "1";
    puts(out.c_str());
}
