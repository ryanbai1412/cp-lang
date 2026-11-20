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

struct Node {
    int mn, cnt;

    Node apply_lazy(int lazy) const {
        return {mn + lazy, cnt};
    }

    Node operator+(const Node& n) const {
        if (mn == n.mn) {
            return {mn, cnt + n.cnt};
        } else if (mn < n.mn) {
            return *this;
        } else {
            return n;
        }
    }

    int count_zeroes() const {
        assert(mn >= 0);
        if (!mn) {
            return cnt;
        } else {
            return 0;
        }
    }
};

struct DS1 {
    int n;
    vector<Node> v;
    vector<int> lazy;

    DS1() {}
    DS1(int n): n(n), v(4 * n, {0, 1}), lazy(4 * n) {}

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o] += x;
            v[o] = v[o].apply_lazy(x);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = (v[lc] + v[rc]).apply_lazy(lazy[o]);
    }

    void update(int ql, int qr, int x) {
        update(1, 0, n - 1, ql, qr, x);
    }

    void update(int ind, int x) {
        update(0, ind, x);
    }

    Node query(int o, int l, int r, int ql, int qr, int clazy) const {
        if (ql <= l && r <= qr) {
            return v[o].apply_lazy(clazy);
        }
        clazy += lazy[o];
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr, clazy) + query(rc, mid + 1, r, ql, qr, clazy);
            }
            return query(lc, l, mid, ql, qr, clazy);
        }
        return query(rc, mid + 1, r, ql, qr, clazy);
    }

    Node query(int l, int r) const {
        return query(1, 0, n - 1, l, r, 0);
    }

    int query(int qr) const {
        int l = 0, r = qr + 1;
        while (l < r) {
            int mid = (l + r) / 2;
            if (query(mid, r).mn >= 0) {
                r = mid;
            } else {
                l = mid + 1;
            }
        }
        if (l > qr) {
            return 0;
        }
        return query(l, qr).count_zeroes();
    }
};

struct DS2 {
    int ind = 0;
    long ans = 0;
    DS1 ds;
    vector<pair<int, long>> st;

    DS2() {}
    DS2(int n) : ds(n) {}

    void push(int x) {
        ds.update(ind, x);
        long d_ans = ds.query(ind);
        ans += d_ans;
        st.emplace_back(x, d_ans);
        ind++;
    }

    void pop() {
        auto [x, d_ans] = st.back();
        st.pop_back();
        ind--;
        ds.update(ind, -x);
        ans -= d_ans;
    }
} ds;

constexpr int maxn = 5e5 + 5;

long ans = 0;
int arr[maxn];
vector<int> graph[maxn];

void dfs(int u) {
    ds.push(-arr[u]);
    ans ^= long(u + 1) * ds.ans;
    for (auto& v : graph[u]) {
        dfs(v);
    }
    ds.pop();
}

void solve() {
    {
        DS1 ds(4);
        ds.update(0, -1);
        ds.update(1, -1);
        ds.update(2, 1);
        dbg(ds.query(1, 2).mn);
    }
    int n;
    cin >> n;
    string s;
    cin >> s;
    for (int i = 0; i < n; i++) {
        if (s[i] == '(') {
            arr[i] = +1;
        } else {
            arr[i] = -1;
        }
    }
    for (int i = 1; i < n; i++) {
        int p;
        cin >> p;
        p--;
        graph[p].push_back(i);
    }
    ds = n;
    dfs(0);
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}