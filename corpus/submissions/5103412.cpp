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

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

constexpr int maxn = 55;

int n, m, ce;
vector<int> arr[maxn];
vector<bool> arrb[maxn];
vector<pair<int, int>> ans;

#ifdef LOCAL
#undef assert
void assert(bool b) {
    if (!b) {
        bool* _asan = new bool[1];
        bool tmp = _asan[1412];
        exit(1);
    }
}
#endif

void dbg_arrb() {
    for (int i = 0; i <= n; i++) {
        cerr << arrb[i] << endl;
    }
}

void move(int u, int v, int cnt = 1) {
    dbg(u, v, cnt);
    assert(u != v);
    for (int i = 0; i < cnt; i++) {
        assert(sz(arr[u]));
        assert(sz(arr[v]) + 1 <= m);
        ans.emplace_back(u, v);
        arr[v].push_back(arr[u].back());
        arr[u].pop_back();
        arrb[v].push_back(arrb[u].back());
        arrb[u].pop_back();
    }
}

void dfs(vector<int> cols, int ql, int qr) {
    dbg(ql, qr);
    if (ql == qr) {
        return;
    }
    int qmid = (ql + qr) / 2;
    for (int i = 0; i <= n; i++) {
        arrb[i].resize(sz(arr[i]));
        for (int j = 0; j < sz(arr[i]); j++) {
            arrb[i][j] = arr[i][j] <= qmid;
        }
    }
    for (auto& a : cols) {
        dbg_arrb();
        int c0 = !a, c1 = ce,
            cnt0 = int(count(begin(arrb[a]), end(arrb[a]), false)),
            cnt1 = m - cnt0;
        move(c0, c1, cnt0);
        dbg(cnt0, cnt1);
        for (int i = sz(arr[a]) - 1; i >= 0; i--) {
            if (arrb[a][i]) {
                move(a, c1);
            } else {
                move(a, c0);
            }
        }
        if (cnt0 > cnt1) {
            move(c0, a, cnt0);
            move(c1, a, cnt1);
        } else {
            move(c1, a, cnt1);
            move(c0, a, cnt0);
        }
        move(c1, c0, cnt0);
    }
    int cnt0[n + 1], cnt1[n + 1];
    auto upd_cnt = [&](int i) -> void {
        cnt0[i] = int(count(begin(arrb[i]), end(arrb[i]), false));
        cnt1[i] = m - cnt0[i];
    };
    auto finished = [&](int i) -> bool {
        return !cnt0[i] || !cnt1[i];
    };
    for (auto& a : cols) {
        upd_cnt(a);
    }
    while (true) {
        bool changed = false;
        for (auto& a : cols) {
            for (auto& b : cols) {
                if (a == b || finished(a) || finished(b) ||
                    arrb[a][0] != arrb[b][0]) {
                    continue;
                }
                changed = true;
                int *ccnt0 = cnt0, *ccnt1 = cnt1;
                if (arrb[a][0]) {
                    swap(ccnt0, ccnt1);
                }
                move(a, ce, ccnt1[a]);
                move(b, ce, ccnt1[b]);
                move(b, a, m - ccnt0[a]);
                int leftc0 = ccnt0[a] + ccnt0[b] - m,
                    leftc1 = ccnt1[a] + ccnt1[b];
                if (leftc0 > leftc1) {
                    move(ce, b, leftc1);
                } else {
                    move(b, ce, leftc0);
                    swap(b, ce);
                }
                upd_cnt(a);
                upd_cnt(b);
            }
        }
        if (!changed) {
            break;
        }
    }
    vector<int> unfinished;
    for (auto& a : cols) {
        if (!finished(a)) {
            unfinished.push_back(a);
        }
    }
    if (sz(unfinished)) {
        assert(sz(unfinished) == 2);
        int a = unfinished[0], b = unfinished[1];
        assert(cnt0[a] == cnt1[b]);
        if (arrb[a][0]) {
            swap(a, b);
        }
        move(a, ce, cnt1[a]);
        move(b, a, cnt0[b]);
        move(ce, b, cnt1[a]);
    }
    vector<int> cl, cr;
    for (auto& a : cols) {
        if (arrb[a][0]) {
            cr.push_back(a);
        } else {
            cl.push_back(a);
        }
    }
    dfs(cl, ql, qmid);
    dfs(cr, qmid + 1, qr);
}

void solve() {
    cin >> n >> m;
    for (int i = 0; i < n; i++) {
        arr[i].resize(m);
        for (auto& a : arr[i]) {
            cin >> a;
            a--;
        }
    }
    vector<int> cols(n);
    iota(begin(cols), end(cols), 0);
    ce = n;
    dfs(cols, 0, n - 1);
    cout << sz(ans) << endl;
    for (auto& [u, v] : ans) {
        cout << u + 1 << " " << v + 1 << endl;
    }
    for (auto& a : arr) {
        if (sz(a)) {
            for (auto& b : a) {
                assert(b == a[0]);
            }
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}