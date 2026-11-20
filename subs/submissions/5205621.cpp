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

constexpr int maxn = 105, maxm = 1e6 + 5;

template <int MOD>
int _rem(string_view s) {
    long carry = 0;
    for (auto& a : s) {
        carry = carry * 10 + a - '0';
        carry %= MOD;
    }
    return int(carry);
}

template <int MOD>
int rem(const string& s) {
    if (s[0] == '-') {
        int ans = _rem<MOD>(string_view(s).substr(1));
        if (!ans) {
            return ans;
        }
        return MOD - ans;
    }
    return _rem<MOD>(s);
}

int n, m;
string arr[maxn];
bool ans[maxm];

template <int MOD>
void prune() {
    int carr[n + 1];
    for (int i = 0; i <= n; i++) {
        carr[i] = rem<MOD>(arr[i]);
    }
    for (int i = 1; i <= m; i++) {
        long cx = 0, base = 1;
        for (int j = 0; j <= n; j++) {
            cx = (cx + base * carr[j]) % MOD;
            base = (base * i) % MOD;
        }
        ans[i] &= !cx;
    }
}

void solve() {
    cin >> n >> m;
    for (int i = 0; i <= n; i++) {
        cin >> arr[i];
        dbg(arr[i]);
    }
    memset(ans, 1, sizeof(ans));
    prune<int(1e9 + 7)>();
    prune<int(1e9 + 9)>();
    prune<998244353>();
    int tcnt = 0;
    for (int i = 1; i <= m; i++) {
        tcnt += ans[i];
    }
    cout << tcnt << endl;
    for (int i = 1; i <= m; i++) {
        if (ans[i]) {
            cout << i << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}