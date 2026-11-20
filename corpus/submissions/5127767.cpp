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

struct DS1 {
    int n;
    vector<int> arr;

    DS1() {}
    DS1(int n) : n(n), arr(n) {}

    void update(int ind, int x) {
        for (int i = 0; i <= ind; i++) {
            arr[i] += x;
        }
    }

    int query(int r) const {
        int ans = 0;
        for (int i = r; i >= 0; i--) {
            if (arr[i] < 0) {
                return ans;
            }
            ans += !arr[i];
        }
        return ans;
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
        DS2 ds(10);
        ds.push(-1);
        dbg(ds.ds.arr);
        ds.push(-1);
        ds.pop();
        dbg(ds.ds.arr);
        ds.push(1);
        dbg(ds.ans);
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