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

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

vector<pair<int, int>> min_sort(vector<int> arr) {
    int n = sz(arr);

    vector<pair<int, int>> ans;

    for (int i = 0; i < n; i++) {
        if (arr[i] == i) {
            continue;
        }
        int u = int(find(begin(arr), end(arr), i) - arr.begin());
        ans.emplace_back(i, u);
        swap(arr[i], arr[u]);
    }

    return ans;
}

vector<pair<int, int>> solve(vector<int> arr,
                             const vector<pair<int, int>>& swaps) {
    int n = sz(arr);

    vector<int> res(n), pos(n);

    for (int i = 0; i < n; i++) {
        pos[arr[i]] = i;
    }

    {
        auto tmp = arr;
        for (auto& [u, v] : swaps) {
            swap(tmp[u], tmp[v]);
        }
        for (int i = 0; i < n; i++) {
            res[tmp[i]] = i;
        }
    }

    auto ans_swaps = min_sort(res);
    dbg(ans_swaps);
    assert(sz(ans_swaps) < sz(swaps));
    vector<pair<int, int>> ans;

    auto do_swap = [&](int u, int v, bool swap_res) -> void {
        swap(arr[u], arr[v]);
        swap(pos[arr[u]], pos[arr[v]]);
        if (swap_res) {
            swap(res[arr[u]], res[arr[v]]);
        }
    };

    for (int t = 0; t < sz(ans_swaps); t++) {
        {
            auto& [u, v] = swaps[t];
            do_swap(u, v, false);
        }
        {
            auto& [u, v] = ans_swaps[t];
            ans.emplace_back(pos[u], pos[v]);
            do_swap(pos[u], pos[v], true);
        }
    }
    for (int t = sz(ans_swaps); t < sz(swaps); t++) {
        ans.emplace_back(0, 0);
    }

    return ans;
}

int findSwapPairs(int n,
                  int arr[],
                  int m,
                  int swaps_x[],
                  int swaps_y[],
                  int out_p[],
                  int out_q[]) {
    vector<pair<int, int>> swaps(m);
    for (int i = 0; i < m; i++) {
        swaps[i] = {swaps_x[i], swaps_y[i]};
    }

    auto ans = solve(vector<int>(arr, arr + n), swaps);

    for (int i = 0; i < sz(ans); i++) {
        auto& [p, q] = ans[i];
        out_p[i] = p;
        out_q[i] = q;
    }

    return sz(ans);
}