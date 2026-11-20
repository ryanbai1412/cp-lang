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

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int(size(x))

ll max_weights(int n, vector<array<int, 3>> fish) {
    vector<pair<int, long>> arr[n];
    for (auto& [x, y, w] : fish) {
        arr[x].emplace_back(y, w);
    }
    for (auto& a : arr) {
        sort(begin(a), end(a));
        long psum = 0;
        for (auto& [_, x] : a) {
            psum += x;
            x = psum;
        }
    }
    auto ibs = [&](int i) -> bool {
        return 0 <= i && i < n;
    };
    auto psum = [&](int i, int x) -> long {
        if (!ibs(i)) {
            return 0;
        }
        auto it =
            lower_bound(begin(arr[i]), end(arr[i]), pair<int, long> {x, 0});
        if (it == arr[i].begin()) {
            return 0;
        }
        return (--it)->second;
    };
    auto sum = [&](int i, int l, int r) -> long {
        return max(long(0), psum(i, r) - psum(i, l));
    };
    auto get = [&](int i, int j) -> int {
        if (!ibs(i)) {
            return 0;
        }
        if (j < sz(arr[i])) {
            return arr[i][j].first;
        } else {
            return n;
        }
    };
    auto mx = [&](const vector<long>& arr) -> long {
        return *max_element(begin(arr), end(arr));
    };
    vector<long> dp[n + 1][2];
    dp[n][0] = dp[n][1] = {0};
    for (int i = n - 1; i >= 0; i--) {
        int m = sz(arr[i]), m2 = sz(dp[i + 1][0]);
        {
            dp[i][0].resize(m + 1, max(mx(dp[i + 1][1]), mx(dp[i + 1][0])));
            long cmx = 0;
            for (int k = 0; k < m2; k++) {
                cmx = max(cmx, dp[i + 1][0][k] - psum(i + 1, get(i + 1, k)));
            }
            for (int j = 0; j <= m; j++) {
                dp[i][0][j] = max(dp[i][0][j], cmx + psum(i + 1, get(i, j)));
            }
            //            for (int j = 0; j <= m; j++) {
            //                for (int k = 0; k < sz(dp[i + 1][0]); k++) {
            //                    dp[i][0][j] =
            //                        max({dp[i][0][j], dp[i + 1][1][k],
            //                             dp[i + 1][0][k] +
            //                                 sum(i + 1, get(i + 1, k), get(i,
            //                                 j))});
            //                }
            //            }
        }
        {
            dp[i][1].resize(m + 1, mx(dp[i + 1][1]));
            {
                long cmx = 0;
                for (int k = 0; k < m2; k++) {
                    cmx = max(cmx, dp[i + 1][1][k] + psum(i, get(i + 1, k)));
                }
                for (int j = 0; j <= m; j++) {
                    dp[i][1][j] = max(dp[i][1][j], cmx - psum(i, get(i, j)));
                }
            }
            {
                long cmx = 0;
                for (int k = 0; k < m2; k++) {
                    cmx =
                        max(cmx, dp[i + 1][0][k] - psum(i + 1, get(i + 1, k)));
                }
                for (int j = 0; j <= m; j++) {
                    dp[i][1][j] =
                        max(dp[i][1][j], cmx + psum(i + 1, get(i, j)));
                }
            }
            //            for (int j = 0; j <= m; j++) {
            //                for (int k = 0; k < sz(dp[i + 1][0]); k++) {
            //                    long x = max(
            //                        dp[i + 1][1][k],
            //                        dp[i + 1][0][k] + sum(i + 1, get(i + 1,
            //                        k), get(i, j)));
            //                    dp[i][1][j] =
            //                        max(dp[i][1][j], x + sum(i, get(i, j),
            //                        get(i + 1, k)));
            //                }
            //            }
        }
    }
    return mx(dp[0][1]);
}

ll max_weights(int n, int m, vector<int> xs, vector<int> ys, vector<int> ws) {
    vector<array<int, 3>> arr;
    for (int i = 0; i < m; i++) {
        arr.push_back({xs[i], ys[i], ws[i]});
    }
    return max_weights(n, arr);
}