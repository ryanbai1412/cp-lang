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

#define endl "\n"
#define long long long int
#define sz(x) int((x).size())

const int bsize = 400, maxn = 1e5 + 5, maxb = maxn / bsize + 2;

struct DS {
    int n;
    vector<int> arr;
    vector<long> sorted[maxb], spsum[maxb];
    vector<pair<long, int>> gaps[maxb];

    DS(const vector<int>& arr): n(sz(arr)), arr(arr) {
        for (int i = 0, ind = 0; ind < sz(arr); i++, ind += bsize) {
            sorted[i] = vector<long>(arr.begin(), arr.begin() + ind);
            sort(begin(sorted[i]), end(sorted[i]), greater<>());
            sorted[i].resize(n + 10, 0);
            spsum[i].resize(sz(sorted[i]) + 1);
            partial_sum(begin(sorted[i]), end(sorted[i]), spsum[i].begin() + 1);
            for (int j = ind; j < min(sz(arr), ind + bsize); j++) {
                gaps[i].emplace_back(arr[j], j);
            }
            sort(begin(gaps[i]), end(gaps[i]), greater<>());
        }
    }

    long query(int i, int k) {
        if (k < 0) {
            return 0;
        }
        int ind = min(k, i + 1), bucket = i / bsize;
        long ans = 0;
        for (auto& [x, j] : gaps[bucket]) {
            if (!ind) {
                break;
            }
            if (j <= i) {
                if (x > sorted[bucket][ind]) {
                    ind--;
                    ans += x;
                } else {
                    break;
                }
            }
        }
        return ans + spsum[bucket][ind];
    }
};

int m;

struct Pref {
    int mul;
    vector<int> arr;
    DS ds;
    vector<long> ans;

    Pref(int mul, const vector<int>& arr)
        : mul(mul), arr(arr), ds(arr), ans(m + 1) {}

    void dp(int l, int r, int ql, int qr) {
        if (l > r) {
            return;
        }
        int mid = (l + r) / 2;
        pair<long, int> opt {-1, -1};
        for (int i = ql; i <= qr; i++) {
            opt = max(opt,
                      pair<long, int> {ds.query(i, mid - (i + 1) * mul), -i});
        }
        auto& [cans, qmid] = opt;
        qmid = -qmid;
        ans[mid] = cans;
        dp(l, mid - 1, ql, qmid);
        dp(mid + 1, r, qmid, qr);
    }

    vector<long> solve() {
        if (sz(arr)) {
            dp(0, m, 0, sz(arr) - 1);
        }
        return ans;
    }
};

template <typename T>
T reversed(T t) {
    reverse(begin(t), end(t));
    return t;
}

long solve(int start, const vector<int>& arr) {
    auto suff =
        Pref(2, vector<int>(arr.begin() + start + 1, arr.end())).solve();
    auto pref = Pref(1, reversed(vector<int>(arr.begin(), arr.begin() + start)))
                    .solve();
    long ans = 0;
    for (int i = 0; i < 2; i++) {
        for (int j = 0; j <= m; j++) {
            int k = m - j - i;
            if (k >= 0) {
                ans = max(ans, i * arr[start] + suff[j] + pref[k]);
            }
        }
    }
    return ans;
}

long findMaxAttraction(int n, int start, int _m, int inarr[]) {
    m = _m;
    vector<int> arr(inarr, inarr + n);
    return max(solve(start, arr), solve(n - start - 1, reversed(arr)));
}