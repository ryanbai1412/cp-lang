#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

struct PSA {
    vector<long> psum;

    PSA() {}
    PSA(const vector<long>& arr) : psum(sz(arr) + 1) {
        partial_sum(begin(arr), end(arr), psum.begin() + 1);
    }

    long query(int l, int r) const {
        return psum[r] - psum[l];
    }
};

constexpr int MAXN = 30, MAXD = 15;

int n;
vector<long> arr;
PSA p_arr;

int cm_t;
int memo1_t[MAXN][MAXN], memo2_t[MAXN][MAXN][MAXN][MAXN];
array<long, MAXD> memo1[MAXN][MAXN];
array<long, MAXD> memo2[MAXN][MAXN][MAXN][MAXN];

const array<long, MAXD>& dp1(int ql, int qr);
const array<long, MAXD>& dp2(int ql1, int qr1, int ql2, int qr2);

long g_w(int ql, int qr) {
    return p_arr.query(ql, qr);
}

long g_w(int ql1, int qr1, int ql2, int qr2) {
    return g_w(ql1, qr1) + g_w(ql2, qr2);
}

void trans(array<long, MAXD>& me,
           pair<long, const array<long, MAXD>*> v1,
           pair<long, const array<long, MAXD>*> v2) {
    long add = min(v1.first, v2.first);
    for (int d = 1; d < MAXD; d++) {
        me[d] = min(me[d], add + (*v1.second)[d - 1] + (*v2.second)[d - 1]);
    }
}

void post_process(array<long, MAXD>& me) {
    for (int d = 1; d < MAXD; d++) {
        me[d] += long(1) << d;
        me[d] = min(me[d], me[d - 1]);
    }
}

pair<long, const array<long, MAXD>*> g_t(int ql, int qr) {
    return {g_w(ql, qr), &dp1(ql, qr)};
}

pair<long, const array<long, MAXD>*> g_t(int ql1, int qr1, int ql2, int qr2) {
    return {g_w(ql1, qr1, ql2, qr2), &dp2(ql1, qr1, ql2, qr2)};
}

const array<long, MAXD>& dp1(int ql, int qr) {
    auto& ans = memo1[ql][qr];
    if (memo1_t[ql][qr] == cm_t) {
        return ans;
    }
    memo1_t[ql][qr] = cm_t;

    fill(begin(ans), end(ans), 1e18);
    if (qr - ql == 0) {
        return ans;
    }
    if (qr - ql == 1) {
        fill(begin(ans), end(ans), 1);
        return ans;
    }

    for (int m1 = ql; m1 <= qr; m1++) {
        for (int m2 = m1; m2 <= qr; m2++) {
            for (int m3 = m2; m3 <= qr; m3++) {
                trans(ans, g_t(ql, m1, m2, m3), g_t(m1, m2, m3, qr));
            }
        }
    }

    if (!(ql == 0 && qr == n)) {
        post_process(ans);
    }

    return ans;
}

const array<long, MAXD>& dp2(int ql1, int qr1, int ql2, int qr2) {
    if (ql1 > ql2) {
        swap(ql1, ql2);
        swap(qr1, qr2);
    }
    assert(qr1 <= ql2);
    if (qr1 == ql2) {
        return dp1(ql1, qr2);
    }
    if (ql1 == qr1) {
        return dp1(ql2, qr2);
    }
    if (ql2 == qr2) {
        return dp1(ql1, qr1);
    }

    auto& ans = memo2[ql1][qr1][ql2][qr2];
    if (memo2_t[ql1][qr1][ql2][qr2] == cm_t) {
        return ans;
    }
    memo2_t[ql1][qr1][ql2][qr2] = cm_t;

    fill(begin(ans), end(ans), 1e18);

    trans(ans, g_t(ql1, qr1), g_t(ql2, qr2));

    for (int m = ql1 + 1; m < qr1; m++) {
        trans(ans, g_t(ql1, m), g_t(m, qr1, ql2, qr2));
        trans(ans, g_t(m, qr1), g_t(ql1, m, ql2, qr2));
    }

    for (int m1 = ql1 + 1; m1 < qr1; m1++) {
        for (int m2 = ql2 + 1; m2 < qr2; m2++) {
            trans(ans, g_t(ql1, m1, ql2, m2), g_t(m1, qr1, m2, qr2));
            trans(ans, g_t(ql1, m1, m2, qr2), g_t(m1, qr1, ql2, m2));
        }
    }

    for (int m1 = ql1; m1 <= qr1; m1++) {
        for (int m2 = qr2; m2 <= qr2; m2++) {
            trans(ans, g_t(ql1, m1, ql2, m2), g_t(m1, qr1, m2, qr2));
            trans(ans, g_t(ql1, m1, m2, qr2), g_t(m1, qr1, ql2, m2));
        }
    }

    for (int m1 = ql1; m1 <= qr1; m1++) {
        for (int m2 = m1; m2 <= qr1; m2++) {
            trans(ans, g_t(ql1, m1, m2, qr1), g_t(m1, m2, ql2, qr2));
        }
    }

    for (int m1 = ql2; m1 <= qr2; m1++) {
        for (int m2 = m1; m2 <= qr2; m2++) {
            trans(ans, g_t(ql2, m1, m2, qr2), g_t(ql1, qr1, m1, m2));
        }
    }

    post_process(ans);

    return ans;
}

long solve(const vector<long>& _arr) {
    n = sz(_arr);
    p_arr = arr;

    if (n == 1) {
        return 0;
    }

    cm_t++;

    auto d_ans = dp1(0, n);
    long ans = *min_element(begin(d_ans), end(d_ans));

    return ans - 2 * (n - 1);
}

void solve() {
    cin >> n;
    arr.resize(n);
    for (auto& a : arr) {
        cin >> a;
    }
    sort(begin(arr), end(arr));

    long ans = 1e18;
    auto scramble = [&](const vector<long>& arr) -> vector<long> {
        vector<long> c;
        int l = 0, r = n;
        for (int i = 0; l < r; i++) {
            if (i & 1) {
                c.push_back(arr[l++]);
            } else {
                c.push_back(arr[--r]);
            }
        }
        return c;
    };
    for (int it = 0; it < 1; it++) {
        arr = scramble(arr);
        ans = min(ans, solve(arr));
    }

    cout << ans << endl;
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