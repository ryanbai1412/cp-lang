#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr double eps = 1e-6;

vector<double> mul(const vector<double>& arra, const vector<double>& arrb) {
    int n = sz(arra), m = sz(arrb);
    vector<double> ans(n + m + 1);
    vector<int> sa[2], sb[2];
    for (int i = 0; i < n; i++) {
        if (arra[i] != 0) {
            sa[arra[i] > eps].push_back(i);
        }
    }
    for (int i = 0; i < m; i++) {
        if (arrb[i] != 0) {
            sb[arrb[i] > eps].push_back(i);
        }
    }
    auto go = [&](const vector<int>& ia, const vector<int>& ib) -> void {
        for (auto& a : ia) {
            for (auto& b : ib) {
                ans[a + b] += arra[a] * arrb[b];
            }
        }
    };
    go(sa[0], sb[1]);
    go(sa[1], sb[0]);
    go(sa[1], sb[1]);
    dbg(sz(sa[0]), sz(sa[1]), sz(sb[0]), sz(sb[1]));
    return ans;
}

vector<double> bpow(vector<double> base, long exp) {
    vector<double> ans {1};
    while (exp) {
        if (exp & 1) {
            ans = mul(ans, base);
        }
        exp >>= 1;
        if (exp) {
            base = mul(base, base);
        }
    }
    return ans;
}

void solve() {
    int m, n, q = 10;
    cin >> m >> n;
    vector<double> base(m, double(1) / m);
    auto dp = bpow(base, n);
    double dpsum[sz(dp) + 1];
    dpsum[0] = 0;
    partial_sum(begin(dp), end(dp), dpsum + 1);
    while (q--) {
        int ql, qr;
        cin >> ql >> qr;
        double ans = dpsum[qr + 1] - dpsum[ql];
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(6);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}