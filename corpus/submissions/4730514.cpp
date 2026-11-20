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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

struct Line {
    long m, b;
    int cnt;

    pair<long, int> operator()(long x) const {
        return {m * x + b, cnt};
    }

    friend ostream& operator<<(ostream& out, const Line& l) {
        return out << "Line {" << l.m << ", " << l.b << ", " << l.cnt << "}";
    }
};

struct DS {
    static const int maxn = 1 << 20;
    Line v[maxn << 1];

    void clear() {
        fill(begin(v), end(v), Line {0, long(4e18), 0});
    }

    void update(int o, int l, int r, Line x) {
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (x(mid) < v[o](mid)) {
            swap(v[o], x);
        }
        if (l == r) {
            return;
        }
        if (x(l) < v[o](l)) {
            update(lc, l, mid, x);
        } else {
            update(rc, mid + 1, r, x);
        }
    }

    void update(Line x) {
        update(1, 0, maxn - 1, x);
    }

    pair<long, int> query(int o, int l, int r, int ind) {
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        auto ans = v[o](ind);
        if (l == r) {
            return ans;
        }
        if (ind <= mid) {
            ans = min(ans, query(lc, l, mid, ind));
        } else {
            ans = min(ans, query(rc, mid + 1, r, ind));
        }
        return ans;
    }

    pair<long, int> query(int ind) {
        dbg(ind);
        return query(1, 0, maxn - 1, ind);
    }
} ds;

int n;
vector<pair<int, int>> arr;

long sq(long x) {
    return x * x;
}

pair<long, int> solve(long x) {
    ds.clear();
    pair<long, int> dp[n + 1];
    auto upd = [&](int i) -> void {
        int r = arr[i - 1].second;
        ds.update({-2 * (r + 1), dp[i].first + sq(r + 1), dp[i].second});
    };
    dp[n] = {0, 0};
    upd(n);
    for (int i = n - 1; i >= 0; i--) {
        auto [cans, cnt] = ds.query(arr[i].first);
        dp[i] = {cans + x + sq(arr[i].first), cnt + 1};
        if (i && arr[i].first <= arr[i - 1].second) {
            dp[i].first -= sq(arr[i - 1].second - arr[i].first + 1);
        }
        if (i) {
            upd(i);
        }
        dbg(dp[i].first, dp[i].second);
    }
    return dp[0];
}

ll take_photos(int _n, int, int k, vector<int> a1, vector<int> a2) {
    n = _n;
    for (int i = 0; i < n; i++) {
        arr.emplace_back(min(a1[i], a2[i]), max(a1[i], a2[i]));
    }
    sort(begin(arr), end(arr), [&](pair<int, int> a, pair<int, int> b) -> bool {
        if (a.first == b.first) {
            return a.second > b.second;
        }
        return a.first < b.first;
    });
    vector<pair<int, int>> narr;
    int mx = -1;
    for (auto& [l, r] : arr) {
        if (r > mx) {
            mx = r;
            narr.emplace_back(l, r);
        }
    }
    swap(arr, narr);
    n = sz(arr);
    for (int i = 0; i < n - 1; i++) {
        assert(arr[i].first < arr[i + 1].first &&
               arr[i].second < arr[i + 1].second);
    }
    auto bsearch = [&](long l, long r) -> long {
        while (l < r) {
            long mid = (l + r) / 2;
            if (solve(mid).second <= k) {
                r = mid;
            } else {
                l = mid + 1;
            }
        }
        return l;
    };
    long l = bsearch(0, 1e9);
    if (l == 1e9) {
        l = bsearch(1e9, 1e18);
    }
    auto [x, cnt] = solve(l);
    dbg(l, x, cnt);
    return x - k * l;
}

ll take_photos(int n, int m, int k, int* a1, int* a2) {
    return take_photos(n, m, k, vector<int>(a1, a1 + n),
                       vector<int>(a2, a2 + n));
}