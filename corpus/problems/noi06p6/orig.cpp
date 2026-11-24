// NOI 2006 The Magical Bag reference. A Polya urn is exchangeable, so only the order of the
// asked draws matters: P = prod_k (a[y_k] + d * earlier draws of y_k) / (S + d * (k - 1)).
// Every factor is below 2e7, so both products are kept as prime exponents and multiplied
// out in base 1e9 once the common primes cancel.
#include <bits/stdc++.h>
using namespace std;

int main() {
    int t, n, d;
    scanf("%d %d %d", &t, &n, &d);
    vector<long long> a(t + 1);
    long long s = 0;
    for (int i = 1; i <= t; i++) scanf("%lld", &a[i]), s += a[i];
    const int LIM = 20'000'001;
    vector<int> spf(LIM);
    for (int i = 2; i < LIM; i++)
        if (!spf[i])
            for (int j = i; j < LIM; j += i)
                if (!spf[j]) spf[j] = i;
    map<int, int> e;
    auto add = [&](long long v, int sign) {
        while (v > 1) { e[spf[v]] += sign; v /= spf[v]; }
    };
    for (int k = 0; k < n; k++) {
        int x, y;
        scanf("%d %d", &x, &y);
        add(a[y], 1), add(s, -1);
        a[y] += d, s += d;
    }
    auto power = [&](int sign) {
        vector<long long> big{1};
        for (auto [p, c] : e)
            for (int i = 0; i < c * sign; i++) {
                long long carry = 0;
                for (auto& w : big) { w = w * p + carry; carry = w / 1'000'000'000; w %= 1'000'000'000; }
                while (carry) big.push_back(carry % 1'000'000'000), carry /= 1'000'000'000;
            }
        string out = to_string(big.back());
        for (int i = (int)big.size() - 2; i >= 0; i--) {
            char buf[16];
            snprintf(buf, sizeof buf, "%09lld", big[i]);
            out += buf;
        }
        return out;
    };
    printf("%s/%s\n", power(1).c_str(), power(-1).c_str());
}
