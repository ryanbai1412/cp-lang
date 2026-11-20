#include <algorithm>
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

struct DS1 {
    struct Maint {
        int me_col;
        const vector<array<int, 3>>& arr;
        set<pair<int, int>> diff[3];

        Maint(int me_col, const vector<array<int, 3>>& arr)
            : me_col(me_col), arr(arr) {}

        const set<pair<int, int>>& get_me() const {
            assert(me_col != -1);
            return diff[me_col];
        }

        int get_val(int ind, int col) {
            if (col == me_col || me_col == -1) {
                return arr[ind][col];
            }
            return arr[ind][col] - arr[ind][me_col];
        }

        void insert(int ind) {
            for (int col = 0; col < 3; col++) {
                diff[col].emplace(get_val(ind, col), ind);
            }
        }

        void erase(int ind) {
            for (int col = 0; col < 3; col++) {
                diff[col].erase({get_val(ind, col), ind});
            }
        }

        int size() const {
            return sz(diff[0]);
        }
    };

    long c_flow = 0;
    array<int, 3> cap;
    vector<array<int, 3>> arr;
    Maint used[3], unused;

    DS1(const array<int, 3>& cap, const vector<array<int, 3>>& arr)
        : cap(cap),
          arr(arr),
          used {Maint(0, arr), Maint(1, arr), Maint(2, arr)},
          unused(-1, arr) {
        for (int i = 0; i < sz(arr); i++) {
            unused.insert(i);
        }
    }

    Maint& get_maint(int c) {
        if (c == -1) {
            return unused;
        }
        return used[c];
    }

    int get_val(int ind, int col) {
        if (col == -1) {
            return 0;
        }
        return arr[ind][col];
    }

    void m_move(int ind, int c1, int c2) {
        dbg("MOVE", ind, c1, c2);
        if (c1 != -1) {
            cap[c1]++;
        }
        if (c2 != -1) {
            cap[c2]--;
        }

        c_flow += get_val(ind, c2) - get_val(ind, c1);
        get_maint(c1).erase(ind);
        get_maint(c2).insert(ind);
    }

    void erase_worst(int col) {
        m_move(used[col].get_me().begin()->second, col, -1);
    }

    void flow() {
        pair<long, vector<pair<int, int>>> opt {-1, {}};

        int perm[3];
        iota(begin(perm), end(perm), 0);

        do {
            int prv = -1;
            long c_ans = 0;
            bool bad = false;
            vector<pair<int, int>> perm_vis;

            for (auto& a : perm) {
                auto [w, qi] = *get_maint(prv).diff[a].rbegin();

                c_ans += w;
                perm_vis.emplace_back(qi, a);

                if (cap[a]) {
                    break;
                }
                if (!sz(get_maint(a))) {
                    bad = true;
                    break;
                }

                prv = a;
            }

            if (bad || (sz(perm_vis) == 3 && !cap[prv])) {
                continue;
            }

            opt = max(opt, {c_ans, perm_vis});
        } while (next_permutation(begin(perm), end(perm)));

        auto [w, ord] = opt;
        assert(w != -1);

        int prv = -1;
        for (auto& [qi, a] : ord) {
            m_move(qi, prv, a);
            prv = a;
        }
    }
};

void solve() {
    int n, kv1, kv2;
    cin >> n >> kv1 >> kv2;
    int arr[n][2];
    for (int j = 0; j < 2; j++) {
        for (int i = 0; i < n; i++) {
            cin >> arr[i][j];
        }
    }

    vector<array<int, 3>> v_arr;
    for (auto& [x, y] : arr) {
        v_arr.push_back({x, y, x + y});
    }

    DS1 ds({0, 0, kv1}, v_arr);
    for (int i = 0; i < kv1; i++) {
        ds.flow();
    }

    assert((ds.cap == array<int, 3> {0, 0, 0}));
    assert(sz(ds.used[2]) == kv1);

    long ans = ds.c_flow;
    dbg(ans);

    while (sz(ds.used[2]) > kv2 && sz(ds.unused)) {
        ds.erase_worst(2);
        dbg(ds.c_flow);

        ds.cap[2]--;
        ds.cap[0]++;
        ds.cap[1]++;
        dbg(ds.cap[0], ds.cap[1], ds.cap[2]);

        ds.flow();
        dbg(ds.c_flow);
        ds.flow();

        ans = max(ans, ds.c_flow);
        dbg(ds.c_flow);
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