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

constexpr long mod = 998244353;

struct DS {
    deque<long> q;
    long v = 0;

    void push_front(long x) {
        x %= mod;
        q.push_front(x);
        v += x;
    }

    void pop_back() {
        v -= q.back();
        if (v < 0) {
            v += mod;
        }
        q.pop_back();
    }

    void clear() {
        q.clear();
        v = 0;
    }

    long query() const {
        return v % mod;
    }
};

void solve() {
    int n, m, kc, kf;
    cin >> n >> m >> kc >> kf;
    bool arr[n][m];
    for (auto& a : arr) {
        string s;
        cin >> s;
        for (int i = 0; i < m; i++) {
            a[i] = s[i] == '1';
        }
    }
    int hor[n][m + 1] {}, vert[n + 1][m] {};
    for (int i = 0; i < n; i++) {
        for (int j = m - 1; j >= 0; j--) {
            if (!arr[i][j]) {
                hor[i][j] = hor[i][j + 1] + 1;
            }
        }
    }
    for (int i = 0; i < m; i++) {
        for (int j = n - 1; j >= 0; j--) {
            if (!arr[j][i]) {
                vert[j][i] = vert[j + 1][i] + 1;
            }
        }
    }
    long ansc = 0, ansf = 0;
    for (int i = 0; i < m; i++) {
        DS dsc, dsf;
        for (int j = n - 1; j >= 0; j--) {
            if (arr[j][i]) {
                dsc.clear();
                dsf.clear();
                continue;
            }
            long chor = max(0, hor[j][i] - 1);
            ansc += dsc.query() * chor;
            ansf += dsf.query() * chor;
            if (j + 1 < n) {
                long mhor = max(0, hor[j + 1][i] - 1);
                dsc.push_front(mhor);
                dsf.push_front(mhor * max(0, vert[j + 1][i] - 1));
            }
        }
    }
    ansc = (ansc % mod * kc) % mod;
    ansf = (ansf % mod * kf) % mod;
    cout << ansc << " " << ansf << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs, _id;
    cin >> tcs >> _id;
    while (tcs--) {
        solve();
    }
}