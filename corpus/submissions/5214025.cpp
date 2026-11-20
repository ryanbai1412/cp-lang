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
using min_pq = priority_queue<T, vector<T>, greater<T>>;

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
    int n;
    vector<int> cnt;
    set<pair<int, int>> v;

    Alloc(int n) : cnt(n, 2), n(n) {
        for (int i = 0; i < n; i++) {
            v.emplace(2, i);
        }
    }

    int alloc() {
        auto [_, ans] = *v.begin();
        v.erase(v.begin());
        if (--cnt[ans]) {
            v.emplace(cnt[ans], ans);
        }
        return ans;
    }

    void free(int ind) {
        if (ind >= n) {
            return;
        }
        if (cnt[ind]) {
            v.erase({cnt[ind], ind});
        }
        v.emplace(++cnt[ind], ind);
    }

    bool empty() const {
        return !sz(v);
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
            pos[x] = {-1, -1};
            finds.free(ind);
            return;
        }
        pos[x] = {ind, sz(sts[ind])};
        sts[ind].push_back(x);
    }

    void op2(int u, int v) {
        dbg(2, u, v);
        ans.push_back({2, u, v});
        assert(u != v && sts[u][0] == sts[v][0]);
        pos[sts[u][0]] = {-1, -1};
        for (auto& ind : {u, v}) {
            sts[ind].erase(sts[ind].begin());
            for (int i = 0; i < sz(sts[ind]); i++) {
                pos[sts[ind][i]].second = i;
            }
            if (sz(sts[ind]) < 2) {
                finds.free(ind);
            }
        }
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
            in.toggle(arr[j++]);
            for (; j < n && !in.all(); j++) {
                in.toggle(arr[j]);
            }
            if (j < n) {
                spec = arr[j];
            }
            dbg(j, spec);
        }
        int x = arr[i];
        auto [ind, p] = ds.pos[x];
        if (ind == -1) {
            int ins;
            if (x == spec) {
                ins = m - 1;
            } else if (ds.finds.empty()) {
                ins = ds.pos[arr[i + 1]].first;
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