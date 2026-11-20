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

struct DS1 {
    int cnt = 0;
    vector<char> v;

    DS1(int n) : v(n) {}

    void toggle(int ind) {
        v[ind] ^= true;
        if (v[ind]) {
            cnt++;
        } else {
            cnt--;
        }
    }

    bool all() const {
        return cnt == sz(v);
    }
};

struct Alloc {
    vector<int> v;

    Alloc(int n) {
        for (int i = 0; i < n; i++) {
            v.push_back(i);
            v.push_back(i);
        }
    }

    int alloc() {
        int ans = v.back();
        v.pop_back();
        return ans;
    }

    void free(int ind) {
        v.push_back(ind);
    }
};

struct DS2 {
    vector<array<int, 3>> ans;
    vector<pair<int, int>> pos;
    vector<vector<int>> sts;
    Alloc finds;

    DS2(int m, int kv) : pos(kv, {-1, -1}), sts(m), finds(m - 1) {}

    void op1(int ind, int x) {
        dbg(1, ind, x);
        ans.push_back({1, ind, -1});
        if (sz(sts[ind]) && sts[ind].back() == x) {
            sts[ind].pop_back();
            pos[ind] = {-1, -1};
            finds.free(ind);
            return;
        }
        pos[x] = {ind, sz(sts[ind])};
        sts[ind].push_back(x);
    }

    void op2(int u, int v) {
        dbg(2, u, v);
        ans.push_back({2, u, v});
        assert(sts[u][0] == sts[v][0]);
        pos[sts[u][0]] = {-1, -1};
        sts[u].erase(sts[u].begin());
        sts[v].erase(sts[v].begin());
        finds.free(u);
        finds.free(v);
    }
};

void solve() {
    int m, n, kv;
    cin >> m >> n >> kv;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int spec = -1;
    DS1 in(kv);
    DS2 ds(m, kv);
    for (int i = 0, j = 0; i < n; i++) {
        if (i == j) {
            spec = -1;
            for (; j < n && !in.all(); j++) {
                in.toggle(arr[j]);
            }
            if (j < n) {
                spec = arr[j];
            }
        }
        int x = arr[i];
        auto& [ind, p] = ds.pos[x];
        if (ind == -1) {
            int ins;
            if (x == spec) {
                ins = m - 1;
            } else {
                ins = ds.finds.alloc();
            }
            ds.op1(ins, x);
        } else {
            if (p == sz(ds.sts[ind]) - 1) {
                ds.op1(ind, x);
            } else {
                ds.op1(m - 1, x);
                ds.op2(ind, m - 1);
            }
        }
    }
    auto& ans = ds.ans;
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