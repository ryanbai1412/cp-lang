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

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    int arr[m];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int touti[m], tout[kv];
    {
        memset(touti, -1, sizeof(touti));
        int st[kv];
        memset(st, -1, sizeof(st));
        for (int i = 0; i < m; i++) {
            int& x = st[arr[i]];
            if (x != -1) {
                touti[x] = i;
                x = -1;
            } else {
                x = i;
            }
        }
    }
    int loc[kv];
    vector<int> sts[n];
    set<pair<int, int>> sind;
    vector<array<int, 3>> ans;
    auto upd_ins = [&](int ind) -> void {
        if (sz(sts[ind]) < 2) {
            sind.emplace(sz(sts[ind]), ind);
        }
    };
    for (int i = 0; i < n; i++) {
        upd_ins(i);
    }
    auto op1 = [&](int ind, int x) -> void {
        loc[x] = ind;
        dbg(1, ind, x);
        ans.push_back({1, ind});
        sind.erase({sz(sts[ind]), ind});
        if (sz(sts[ind]) && sts[ind].back() == x) {
            sts[ind].pop_back();
        } else {
            sts[ind].push_back(x);
        }
        upd_ins(ind);
    };
    auto op2 = [&](int u, int v) -> void {
        dbg(2, u, v);
        ans.push_back({2, u, v});
        assert(u != v && sz(sts[u]) && sz(sts[v]) && sts[u][0] == sts[v][0]);
        for (auto& a : {u, v}) {
            sind.erase({sz(sts[a]), a});
            sts[a].erase(sts[a].begin());
            upd_ins(a);
        }
    };
    for (int iteri = 0; iteri < m; iteri++) {
        int a = arr[iteri];
        tout[a] = touti[iteri];
        int f2 = -1, cnt2 = 0;
        for (int i = 0; i < n; i++) {
            if (!sz(sts[i])) {
                f2 = i;
                cnt2++;
            }
        }
        if (tout[a] == -1) {
            int i = loc[a];
            assert(sz(sts[i]));
            if (sts[i].back() == a) {
                op1(i, a);
                goto loop;
            } else if (sts[i][0] == a) {
                assert(f2 != -1);
                op1(f2, a);
                op2(i, f2);
                goto loop;
            }
            assert(false);
        }
        if (!cnt2) {
            pair<int, int> mx {-1, -1};
            for (int i = 0; i < n; i++) {
                if (sz(sts[i]) == 1) {
                    int ctout = tout[sts[i][0]];
                    mx = max(mx, {ctout, i});
                }
            }
            dbg(iteri, a, tout[a]);
            assert(mx.second != -1);
            op1(mx.second, a);
            goto loop;
        }
        if (sz(sind)) {
            if (cnt2 > 1) {
                op1(sind.begin()->second, a);
                goto loop;
            } else {
                auto it = sind.lower_bound({1, -1});
                if (it != sind.end()) {
                    op1(it->second, a);
                    goto loop;
                }
            }
        }
        for (int i = 0; i < n; i++) {
            if (sz(sts[i]) == 2) {
                if (tout[sts[i][0]] < tout[sts[i][1]]) {
                    op1(i, a);
                    goto loop;
                }
            }
        }
        op1(f2, a);
    loop:;
    }
    cout << sz(ans) << endl;
    for (auto& [t, a, b] : ans) {
        a++;
        b++;
        if (t == 1) {
            cout << t << " " << a << endl;
        } else {
            cout << t << " " << a << " " << b << endl;
        }
    }
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