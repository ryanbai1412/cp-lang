// Not a judge-accepted submission: written for the corpus and checked against the samples and an exhaustive brute.
#include <bits/stdc++.h>
using namespace std;
using Big = vector<uint32_t>;  // base 1e9, little endian
const uint32_t BASE = 1000000000;
void add(Big& a, const Big& b) {
    uint32_t c = 0;
    for (size_t i = 0; i < max(a.size(), b.size()) || c; i++) {
        if (i == a.size()) a.push_back(0);
        uint64_t cur = (uint64_t)a[i] + c + (i < b.size() ? b[i] : 0);
        a[i] = cur % BASE, c = cur / BASE;
    }
}
int main() {
    int k, w;
    cin >> k >> w;
    int m = (1 << k) - 1, q = w / k, r = w % k;
    // C[a][b] = binomial(a, b) for a <= m
    vector<vector<Big>> C(m + 1, vector<Big>(m + 1));
    for (int a = 0; a <= m; a++) {
        C[a][0] = {1};
        for (int b = 1; b <= a; b++) C[a][b] = C[a - 1][b - 1], add(C[a][b], C[a - 1][b]);
    }
    Big ans;
    // q full k-bit digits: any 2..q distinct nonzero digits in increasing order
    for (int d = 2; d <= min(q, m); d++) add(ans, C[m][d]);
    // a leading r-bit digit t, then q larger digits
    for (int t = 1; r && t < (1 << r) && q >= 1; t++)
        if (m - t >= q) add(ans, C[m - t][q]);
    if (ans.empty()) { puts("0"); return 0; }
    printf("%u", ans.back());
    for (size_t i = ans.size() - 1; i-- > 0;) printf("%09u", ans[i]);
    puts("");
}
