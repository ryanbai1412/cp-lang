#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

struct BDS {
    vector<pair<int, long>> arr;

    void push_front(long x) {
        arr.emplace(arr.begin(), -1e9, x);
    }

    void pop_back() {
        arr.pop_back();
    }

    void update_all(int x) {
        for (auto& [a, _b] : arr) {
            a = max(a, x);
        }
    }

    long query_all() const {
        long ans = 0;

        for (auto& [a, b] : arr) {
            ans = max(ans, a + b);
        }

        return ans;
    }
};

template <typename T>
struct A {
    static constexpr int maxn = 1 << 20;

    inline static T* heap = new T[maxn];
    inline static vector<int> inds;

    A() {
        if (!sz(inds)) {
            for (int i = 0; i < maxn; i++) {
                inds.push_back(i);
            }
        }
    }

    using value_type = T;

    T* allocate(size_t n) {
        assert(n == 1);

        int ind = inds.back();
        inds.pop_back();

        return &heap[ind];
    }

    void deallocate(T* p, size_t n) {
        assert(n == 1);

        inds.push_back(int(p - heap));
    }
};

template <typename T>
using L = list<T, A<T>>;

struct DS2 {
    long accum_add = 0;
    L<pair<long, long>> v;
    L<int> inds;

    void push(int ind, long x) {
        v.emplace_back(0, x);
        inds.push_back(ind);
    }

    int size() const {
        return sz(v);
    }

    void pop_back() {
        accum_add -= v.back().first;
        v.pop_back();
        inds.pop_back();
    }

    void pop_front() {
        long cx = v.front().first;
        v.pop_front();
        if (sz(v)) {
            v.front().first += cx;
        } else {
            accum_add = 0;
        }
        inds.pop_front();
    }

    long front() const {
        return v.front().first + v.front().second;
    }

    long back() const {
        return accum_add + v.back().second;
    }

    void range_add(long x) {
        v.front().first += x;
        accum_add += x;
    }

    void extend(DS2& d) {
        if (sz(d.v)) {
            d.v.front().first -= accum_add;
            accum_add = d.accum_add;
            v.splice(v.end(), d.v);
        }
        inds.splice(inds.end(), d.inds);
    }
};

struct DS {
    // [ql, qr)
    int ql = 0, qr = 0;
    deque<pair<int, DS2>> arr;

    void push_front(long x) {
        DS2 tmp;
        tmp.push(--ql, x - 1e9);

        arr.emplace_front(-1e9, tmp);
    }

    void pop_back() {
        if (arr.back().second.inds.back() != --qr) {
            return;
        }

        arr.back().second.pop_back();

        if (!sz(arr.back().second)) {
            arr.pop_back();
        }
    }

    long query_all() const {
        return arr.back().second.back();
    }

    void update_all(int x) {
        DS2 cd;

        while (sz(arr) && arr.front().first <= x) {
            auto& [cx, cl] = arr.front();

            cl.range_add(x - cx);

            if (sz(cd)) {
                dbg(cd.back(), cl.front());
            }
            while (sz(cl) && sz(cd) && cl.front() <= cd.back()) {
                cl.pop_front();
            }

            cd.extend(cl);
            arr.pop_front();
        }

        dbg(sz(arr));
        if (sz(cd)) {
            while (sz(arr) && arr.front().second.front() <= cd.back()) {
                arr.front().second.pop_front();
                if (!sz(arr.front().second)) {
                    arr.pop_front();
                }
            }

            arr.emplace_front(x, cd);
        }
    }
};

void solve() {
    {
        DS ds;
        ds.update_all(2);
        ds.push_front(15);
        ds.update_all(2);
        ds.push_front(14);
        ds.update_all(4);
        dbg(ds.arr.front().second.front());
        dbg(ds.arr.front().second.back());
        dbg(ds.query_all(), sz(ds.arr.front().second));
        ds.push_front(14);
        dbg("-------");
        ds.update_all(4);
        dbg(ds.query_all());
        // return;
    }
    int n, kv;
    cin >> n >> kv;

    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }

    long dp[n];

    for (int qr = n - 1; qr >= 0; qr -= kv) {
        // [ql, qr]
        int ql = max(0, qr - kv + 1);

        DS ds;
        for (int i = qr + kv; i > qr; i--) {
            if (i >= n) {
                ds.push_front(0);
            } else {
                ds.update_all(arr[i]);
                ds.push_front(dp[i]);
            }
        }

        for (int i = qr; i >= ql; i--) {
            ds.update_all(arr[i]);
            dp[i] = ds.query_all();
            dbg(i, dp[i]);
            ds.pop_back();
        }
    }

    cout << dp[0] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}