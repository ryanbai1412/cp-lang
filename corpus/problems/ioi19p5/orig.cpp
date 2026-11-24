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

// The sample grader, reading the input the port reads (corpus/submissions/5664246.cpp is the grader function).
static vector<pair<int, vector<int>>> grader_gates;  // op 0 and, 1 or, 2 xor, 3 not
static int grader_inputs;
static int grader_add(int op, vector<int> ns) {
    grader_gates.push_back({op, ns});
    return grader_inputs + int(grader_gates.size()) - 1;
}
int add_and(vector<int> ns) { return grader_add(0, ns); }
int add_or(vector<int> ns) { return grader_add(1, ns); }
int add_xor(vector<int> ns) { return grader_add(2, ns); }
int add_not(int n) { return grader_add(3, {n}); }
int main() {
    int h, w, k; scanf("%d %d %d", &h, &w, &k);
    grader_inputs = h * w;
    construct_network(h, w, k);
    int r1, c1, r2, c2;
    while (scanf("%d", &r1) == 1 && r1 != -1) {
        scanf("%d %d %d", &c1, &r2, &c2);
        vector<char> on(h * w, 0);
        on[r1 * w + c1] = on[r2 * w + c2] = 1;
        for (auto& [op, ns] : grader_gates) {
            int v = op == 0;
            for (int i : ns) v = op == 0 ? v & on[i] : op == 1 ? v | on[i] : op == 2 ? v ^ on[i] : !on[i];
            on.push_back(v);
        }
        printf("%d\n", int(on.back()));
    }
}
