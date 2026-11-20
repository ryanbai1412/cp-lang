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

struct Node {
    int mxa, mxb, lastd, cnt;
    long sm;

    Node operator+(const Node& n) const {
        return {max(mxa, n.mxa), max(mxb, n.mxb),
                n.cnt ? n.lastd : lastd, cnt + n.cnt, sm + n.sm};
    }

    static Node def() {
        return {0, 0, 0, 0, 0};
    }

    static Node from(int a, int b) {
        return {a, b, a - b, 1, a - b};
    }
};

struct ST {
    int n;
    vector<Node> arr, v;

    ST(const vector<Node>& arr) : n(sz(arr)), arr(arr), v(4 * n) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = arr[l];
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void update(int o, int l, int r, int ind, const Node& x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = v[lc] + v[rc];
    }

    void update(int ind, const Node& x) {
        update(1, 0, n - 1, ind, x);
    }

    Node query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr) +
                       query(rc, mid + 1, r, ql, qr);
            }
            return query(lc, l, mid, ql, qr);
        }
        return query(rc, mid + 1, r, ql, qr);
    }

    Node query(int l, int r) const {
        if (l > r) {
            return Node::def();
        }
        return query(1, 0, n - 1, l, r);
    }
};

void solve() {
    int n;
    cin >> n;
    vector<array<int, 3>> arr;
    vector<pair<bool, int>> queries;
    for (int i = 0; i < n; i++) {
        int a, b;
        cin >> a >> b;
        arr.push_back({a, b, i});
    }
    int q;
    cin >> q;
    for (int i = 0; i < q; i++) {
        int t;
        cin >> t;
        if (!t) {
            int x;
            cin >> x;
            queries.emplace_back(false, x);
        } else {
            t--;
            int a, b;
            cin >> a >> b;
            arr.push_back({a, b, i + n});
            queries.emplace_back(true, t);
        }
    }
    sort(begin(arr), end(arr), [&](const auto& a, const auto& b) -> bool {
        return a[0] - a[1] > b[0] - b[1];
    });
    int pos;
    for (pos = 0; pos < sz(arr) && arr[pos][0] - arr[pos][1] > 0; pos++);
    int conv1[n + q], carr[n];
    for (int i = 0; i < sz(arr); i++) {
        conv1[arr[i][2]] = i;
    }
    vector<Node> init;
    for (int i = 0; i < sz(arr); i++) {
        auto& [a, b, j] = arr[i];
        if (j < n) {
            carr[j] = i;
            init.push_back(Node::from(a, b));
        } else {
            init.push_back(Node::def());
        }
    }
    ST st(init);
    for (int i = 0; i < q; i++) {
        auto& [t, x] = queries[i];
        if (!t) {
            int l = 0, r = pos + 1;
            while (l < r) {
                int mid = (l + r) / 2;
                Node a = st.query(0, mid - 1), b = st.query(mid, sz(init) - 1);
                dbg(mid, a.sm, a.mxb);
                if (a.sm + a.mxb >= x || a.sm - a.lastd + b.mxa >= x) {
                    r = mid;
                } else {
                    l = mid + 1;
                }
            }
            if (l == pos + 1) {
                Node a = st.query(0, pos - 1), b = st.query(pos, sz(init) - 1);
                if (a.sm + b.mxa >= x) {
                    cout << a.cnt + 1 << endl;
                } else {
                    cout << -1 << endl;
                }
            } else {
                cout << max(1, st.query(0, l - 1).cnt) << endl;
            }
        } else {
            st.update(carr[x], Node::def());
            int j = conv1[i + n];
            st.update(j, Node::from(arr[j][0], arr[j][1]));
            carr[x] = j;
        }
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}