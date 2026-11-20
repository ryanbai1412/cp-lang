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

int add_and(vector<int> Ns);
int add_or(vector<int> Ns);
int add_xor(vector<int> Ns);
int add_not(int N);

void construct_network(int n, int m, int kv) {
    auto id_arr = [&](int x, int y) -> int { return x * m + y; };

    int b_true = -1, b_false = -1;
 
    auto g_true = [&]() -> int {
        if (b_true == -1) {
            vector<int> cq;

            for (int i = 0; i < n; i++) {
                for (int j = 0; j < m; j++) {
                    cq.push_back(id_arr(i, j));
                }
            }

            b_true = add_or(cq);
        }
        return b_true;
    };
    auto g_false = [&]() -> int {
        if (b_false == -1) {
            b_false = add_not(g_true());
        }
        return b_false;
    };

    auto go_cell = [&](auto cb) -> vector<int> {
        map<int, vector<int>> mp;
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < m; j++) {
                mp[cb(i, j)].push_back(id_arr(i, j));
            }
        }
        vector<int> ans;
        for (auto& [_k, v] : mp) {
            ans.push_back(add_or(v));
        }
        return ans;
    };

    auto has_k_diff = [&](const vector<int>& arr, int kv) -> int {
        int n = sz(arr);
        if (kv >= n) {
            return g_false();
        }

        vector<int> cq;

        for (int i = 0; i + kv < n; i++) {
            cq.push_back(add_and({arr[i], arr[i + kv]}));
        }

        return add_or(cq);
    };

    auto has_ge_k_diff = [&](const vector<int>& arr, int kv) -> int {
        int n = sz(arr);
        if (kv >= n) {
            return g_false();
        }

        vector<int> cq;
        int suff = arr[0];

        for (int i = kv; i < n; i++) {
            suff = add_or({suff, arr[i - kv]});
            cq.push_back(add_and({arr[i], suff}));
        }

        return add_or(cq);
    };

    auto c1 = go_cell([&](int x, int y) -> int { return x + y; }),
         c2 = go_cell([&](int x, int y) -> int { return x - y; });

    int has_k = add_or({has_k_diff(c1, kv), has_k_diff(c2, kv)}),
        has_gt_k =
            add_or({has_ge_k_diff(c1, kv + 1), has_ge_k_diff(c2, kv + 1)});

    add_and({has_k, add_not(has_gt_k)});
}