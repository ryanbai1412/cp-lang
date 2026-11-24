// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, m;
    cin >> n >> m;
    vector<long long> a(n);
    long long s = 0;
    for (auto &x : a) cin >> x, s += x;
    long long t = s / m;
    int full = (1 << n) - 1;
    vector<long long> rem(1 << n, 0);
    vector<int> best(1 << n, 0);
    for (int mask = 1; mask <= full; mask++) {
        int i = __builtin_ctz(mask);
        rem[mask] = (rem[mask ^ (1 << i)] + a[i]) % t;
        int b = 0;
        for (int j = 0; j < n; j++)
            if (mask >> j & 1) b = max(b, best[mask ^ (1 << j)]);
        best[mask] = b + (rem[mask] == 0);
    }
    cout << m - best[full] << "\n";
}
