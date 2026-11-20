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

static constexpr long inf = 4e18, minf = 1e18;

struct sint {
    long x;

    sint() : x(0) {}
    sint(long x) : x(min(x, inf)) {}

    friend ostream& operator<<(ostream& out, const sint& s) {
        return out << long(s.x);
    }

    friend sint operator-(const sint& s) {
        return -s.x;
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

    bool operator==(const sint& s) const {
        return x == s.x;
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
    int b;
    sint c;

    sint operator()() const {
        sint cp = 1;
        for (int i = 0; i < p; i++) {
            cp *= b;
        }
        return cp + c;
    }

    friend bool operator<(const Res& r1, const Res& r2) {
        if (r2.c > minf) {
            return true;
        } else if (r1.c > minf) {
            return false;
        }
        auto& [b1, c1] = r1;
        auto& [b2, c2] = r2;
        sint pow1[p + 1], pow2[p + 1];
        pow1[0] = pow2[0] = 1;
        for (int i = 0; i < p; i++) {
            pow1[i + 1] = pow1[i] * b1;
            pow2[i + 1] = pow2[i] * b2;
        }
        sint ans = 0;
        for (int i = 0; i < p; i++) {
            ans += pow1[i] * pow2[p - i - 1];
        }
        if (b1 - b2 < 0) {
            ans *= b2 - b1;
            ans = -ans;
        } else {
            ans *= b1 - b2;
        }
        ans += c1 - c2;
//        fprintf(stderr, "print(%d ** %d + %ld < %d ** %d + %ld, %d);\n", b1, p,
//                c1.x, b2, p, c2.x, ans < 0);
        //        dbg(b1, c1, b2, c2, ans);
        return ans < 0;
    }
};

struct Line {
    int k;
    sint c;
    int data;

    sint operator()(int x) const {
        return eval(x)();
    }

    Res eval(int x) const {
        return {abs(k + x), c};
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
    //    static constexpr int maxn = 1 << 22;
    static constexpr int maxn = 1 << 10;

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
        if (x.eval(mid) < cx.eval(mid)) {
            swap(x, cx);
        }
        if (l == r) {
            return;
        }
        if (x.eval(l) < cx.eval(l)) {
            update(lc, l, mid, x);
        } else {
            update(rc, mid + 1, r, x);
        }
    }

    void insert(const Line& x) {
        dbg("I", x.k, x.c);
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
        dbg("Q", x);
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
        dbg(i, cans);
        dp[i] = cans;
        opt[i] = copt;
        ds.insert({kv - (psum[i] + i), cans, i});
    }
    if (dp[0] > minf) {
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