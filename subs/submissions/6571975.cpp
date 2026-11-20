#include <bits/allocator.h>

#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

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

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

inline void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

template <typename T>
struct PSA {
    int n;
    vector<T> psum;

    template <typename U>
    PSA(const vector<U>& arr) : n(sz(arr)), psum(n + 1) {
        for (int i = 0; i < n; i++) {
            psum[i + 1] = psum[i] + arr[i];
        }
    }

    T query(int l, int r) const {
        return psum[r] - psum[l];
    }
};

template <typename T>
bool on(T mask, int bit) {
    return (mask >> bit) & 1;
}

template <typename T>
vector<T> iota(int n, T x) {
    vector<T> arr(n);
    iota(begin(arr), end(arr), x);
    return arr;
}

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

namespace st1 {

struct Node {
    int lc, rc, v;
} heap[(200 << 20) / sizeof(Node)];
int n, hind = 1;

void update(int& root, int ind, int x) {
    int *o = &root, l = 0, r = n - 1;

    while (true) {
        heap[hind] = heap[*o];
        *o = hind++;

        heap[*o].v++;

        int mid = (l + r) / 2;
        if (ind < mid) {
            o = &heap[*o].lc;
            r = mid - 1;
        } else if (ind > mid) {
            o = &heap[*o].rc;
            l = mid + 1;
        } else {
            break;
        }
    }
}

int query_pref(int o, int qr) {
    int ans = 0, l = 0, r = n - 1;

    while (true) {
        if (!o || qr < l) {
            return ans;
        }

        int mid = (l + r) / 2;
        if (qr < mid) {
            o = heap[o].lc;
            r = mid - 1;
        } else {
            ans += heap[o].v - heap[heap[o].rc].v;
            o = heap[o].rc;
            l = mid + 1;
        }
    }
}

}  // namespace st1

struct Solver {
    int n;
    vector<pair<int, int>> arr;
    vector<int> roots;

    Solver() {}
    Solver(const vector<pair<int, int>>& _arr)
        : n(sz(_arr)), arr(_arr), roots(n + 1) {
        sort(begin(arr), end(arr), CmpByKey([&](const auto& a) -> int {
            return -a.second;
        }));
        st1::n = n + 5;

        // assert(n <= 1e5);
        int root = 0;
        for (int i = n, j = 0; i >= 0; i--) {
            for (; j < n && arr[j].second == i; j++) {
                st1::update(root, arr[j].first, +1);
            }

            roots[i] = root;
        }

        sort(begin(arr), end(arr), CmpByKey([&](const auto& a) -> int {
            return a.first;
        }));
    }

    bool query2(vector<pair<int, int>> carr) {
        int m = sz(carr);

        int va[m + 1][m + 1] {};
        {
            int comp2[n + 10];
            for (int i = 0; i < m; i++) {
                int cr = i + 1 < m ? carr[i + 1].first : n + 5;
                fill(comp2 + max(0, carr[i].first), comp2 + cr, i);
            }

            int j = 0;
            for (auto& [l, r] : arr) {
                for (; j + 1 < m && carr[j + 1].first < l; j++)
                    ;
                va[j + 1][comp2[r]]++;
            }
        }

        for (int i = 1; i < m; i++) {
            for (int j = m - 1; j >= 0; j--) {
                va[i][j] += va[i - 1][j] + va[i][j + 1] - va[i - 1][j + 1];
            }
        }
        int va2[m];
        for (int i = 0; i < m; i++) {
            va2[i] = va[i][i];
        }

        int dp[m] {};

        for (int i = m - 1; i >= 0; i--) {
            for (int j = i + 1; j < m; j++) {
                dp[i] = min(dp[i], -va[i][j] + dp[j]);
            }
            dp[i] += va2[i] - carr[i].second;
        }

        return dp[0] >= 0;
    }

    vector<pair<int, int>> carr;
    bool query(vector<int> qarr) {
        if (accumulate(begin(qarr), end(qarr), long(0)) > n) {
            return false;
        }

        carr.clear();
        sort(begin(qarr), end(qarr));
        for (int i = 0; i < sz(qarr);) {
            int cl = i;
            for (; i < sz(qarr) && qarr[cl] == qarr[i]; i++)
                ;
            carr.emplace_back(qarr[cl], (i - cl) * qarr[cl]);
        }
        carr.insert(begin(carr), pair {-1, 0});

        if (sz(carr) > 250) {
            return query2(carr);
        }
        int m = sz(carr);

        int dp[m] {};
        for (int j = m - 1; j >= 1; j--) {
            int qr = carr[j].first, cv = st1::query_pref(roots[qr], qr);

            for (int i = 0; i < j; i++) {
                dp[i] =
                    min(dp[i], cv - st1::query_pref(roots[qr], carr[i].first) +
                                   dp[j] - carr[j].second);
            }
        }

        return dp[0] >= 0;
    }
} solver;

void init(int n, int arra[], int arrb[]) {
    vector<pair<int, int>> cv(n);

    for (int i = 0; i < n; i++) {
        cv[i] = {arra[i], arrb[i]};
    }

    solver = Solver(cv);
}

int can(int m, int arr[]) {
    return solver.query(vector(arr, arr + m));
}