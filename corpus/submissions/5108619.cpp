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

using ld = long double;
using i128 = __int128_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

static constexpr i128 inf = 1e36;

struct sint {
    i128 x;

    sint() : x(0) {}
    sint(i128 x) : x(min(x, inf)) {}

    friend ostream& operator<<(ostream& out, const sint& s) {
        return out << long(s.x);
    }

    sint& operator+=(const sint& s) {
        x = min(inf, x + s.x);
        return *this;
    }
    sint& operator-=(const sint& s) {
        x = min(inf, x - s.x);
        return *this;
    }
    sint& operator*=(const sint& s) {
        if (s.x && x >= inf / s.x) {
            x = inf;
        } else {
            x *= s.x;
        }
        return *this;
    }

    friend sint operator+(sint a, const sint& b) {
        return a += b;
    }
    friend sint operator-(sint a, const sint& b) {
        return a -= b;
    }
    friend sint operator*(sint a, const sint& b) {
        return a *= b;
    }

    bool operator!=(const sint& s) const {
        return x != s.x;
    }
    bool operator<(const sint& s) const {
        return x < s.x;
    }
    bool operator>(const sint& s) const {
        return x > s.x;
    }

    sint abs() const {
        return max(x, -x);
    }
};

int p;

struct Res {
    sint ans;
    ld ansl;
    int x;
    sint c;

    bool operator<(const Res& r) const {
        return make_tuple(ans, ansl, x, c) < make_tuple(r.ans, r.ansl, r.x, r.c);
    }
};

struct Line {
    int k;
    sint c;
    int data;

    sint operator()(int x) const {
        x = abs(k + x);
        sint cp = 1;
        for (int i = 0; i < p; i++) {
            cp *= x;
        }
        return cp + c;
    }

    Res eval(int x) const {
        x = abs(k + x);
        sint cp = 1;
        ld cpl = 1;
        for (int i = 0; i < p; i++) {
            cp *= x;
            cpl *= x;
        }
        return {cp + c, cpl, x, c};
    }
};

struct BDS {
    vector<Line> arr;

    void insert(const Line& x) {
        arr.push_back(x);
    }

    pair<sint, int> query(int x) const {
        pair<sint, int> ans {inf, -1};
        for (auto& a : arr) {
            ans = min(ans, pair<sint, int> {a(x), a.data});
        }
        return ans;
    }
};

struct FDS {
    static constexpr int maxn = 1 << 22;
    //    static constexpr int maxn = 1 << 10;

    Line v[maxn << 1];

    FDS() {
        fill(begin(v), end(v), Line {0, inf, -1});
    }

    void update(int o, int l, int r, Line x) {
        if (l > r) {
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        Line& cx = v[o];
        if (x(mid) < cx(mid)) {
            swap(x, cx);
        }
        if (l == r) {
            return;
        }
        if (x(l) < cx(l)) {
            update(lc, l, mid, x);
        } else {
            update(rc, mid + 1, r, x);
        }
    }

    void insert(const Line& x) {
        update(1, 0, maxn - 1, x);
    }

    pair<sint, int> query(int o, int l, int r, int x) const {
        pair<sint, int> ans {v[o](x), v[o].data};
        if (l == r) {
            return ans;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (x <= mid) {
            return min(ans, query(lc, l, mid, x));
        } else {
            return min(ans, query(rc, mid + 1, r, x));
        }
    }

    pair<sint, int> query(int x) const {
        return query(1, 0, maxn - 1, x);
    }
};

using DS = FDS;

void solve() {
    int n, kv;
    cin >> n >> kv >> p;
    string sarr[n];
    for (auto& a : sarr) {
        while (!sz(a)) {
            getline(cin, a);
        }
    }
    int arr[n];
    for (int i = 0; i < n; i++) {
        arr[i] = sz(sarr[i]);
        dbg(arr[i]);
    }
    int psum[n + 1];
    psum[0] = 0;
    partial_sum(arr, arr + n, psum + 1);
    DS ds;
    sint dp[n];
    int opt[n];
    ds.insert({kv - (psum[n] + n), 0, n});
    for (int i = n - 1; i >= 0; i--) {
        auto [cans, copt] = ds.query(psum[i] + i + 1);
        dp[i] = cans;
        opt[i] = copt;
        ds.insert({kv - (psum[i] + i), cans, i});
    }
    if (dp[0] > long(1e18)) {
        cout << "Too hard to arrange" << endl;
    } else {
        cout << dp[0] << endl;
        for (int i = 0; i < n;) {
            int j = opt[i];
            for (; i < j; i++) {
                cout << sarr[i] << " \n"[i == j - 1];
            }
        }
    }
    cout << string(20, '-') << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}