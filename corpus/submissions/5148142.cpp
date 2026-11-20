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

using i128 = __int128;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct D {
    static constexpr int maxn = 4e7 + 5;

    long arr1[maxn];
    int arr2[maxn], l = 0, r = 0;

    int size() const {
        return r - l;
    }

    pair<long, int> get(int i) {
        return {arr1[i], arr2[i]};
    }

    pair<long, int> back() {
        return get(r - 1);
    }

    pair<long, int> front() {
        return get(l);
    }

    void push_back(const pair<long, int>& t) {
        arr1[r] = t.first;
        arr2[r++] = t.second;
    }

    void pop_back() {
        r--;
    }

    void pop_front() {
        l++;
    }

    pair<long, int> operator[](int i) {
        return get(l + i);
    }
};

struct DS {
    D arr;

    void insert(long k, int v) {
        while (sz(arr) && arr.back().first >= k) {
            arr.pop_back();
        }
        arr.push_back({k, v});
    }

    int query(long x) {
        while (sz(arr) >= 2 && arr[1].first <= x) {
            arr.pop_front();
        }
        return arr.front().second;
    }
};

i128 sq(i128 x) {
    return x * x;
}

void print(ostream& out, i128 x) {
    if (!x) {
        return;
    }
    print(out, x / 10);
    out << char('0' + x % 10);
}

ostream& operator<<(ostream& out, i128 x) {
    if (x < 0) {
        out << '-';
        return out << -x;
    } else if (x == 0) {
        return out << '0';
    }
    print(out, x);
    return out;
}

void solve(const vector<int>& arr) {
    int n = sz(arr);
    long psum[n + 1];
    psum[0] = 0;
    for (int i = 0; i < n; i++) {
        psum[i + 1] = psum[i] + arr[i];
    }
    DS ds;
    pair<i128, long> dp[n + 1];
    dp[0] = {};
    ds.insert(0, 0);
    for (int i = 1; i <= n; i++) {
        {
            int j = ds.query(psum[i]);
            dp[i] = {dp[j].first - i128(2) * psum[i] * psum[j], -psum[j]};
        }
        dp[i].first += 2 * sq(psum[i]);
        dp[i].second += 2 * psum[i];
        ds.insert(dp[i].second, i);
    }
    cout << dp[n].first - sq(psum[n]) << endl;
}

void solve() {
    int n, tc_type;
    cin >> n >> tc_type;
    vector<int> arr(n);
    if (tc_type == 0) {
        for (auto& a : arr) {
            cin >> a;
        }
    } else {
        int b[n];
        long x, y, z, m;
        cin >> x >> y >> z >> b[0] >> b[1] >> m;
        int bounds[m][3];
        for (auto& [p, l, r] : bounds) {
            cin >> p >> l >> r;
        }
        for (int i = 2; i < n; i++) {
            b[i] = int((x * b[i - 1] + y * b[i - 2] + z) & ((1 << 30) - 1));
        }
        for (int i = 0, j = 0; i < m; i++) {
            auto& [p, l, r] = bounds[i];
            for (; j < p; j++) {
                arr[j] = (b[j] % (r - l + 1)) + l;
            }
        }
    }
    dbg(sizeof(D));
    solve(arr);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}