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

namespace grader {

constexpr int maxn = 4e4 + 5;

int n, arr[maxn], cap[maxn], queries = 0;

void init(int _n) {
    n = _n;
    for (int i = 0; i < n; i++) {
        cin >> cap[i];
    }
    copy(cap, cap + n, arr);
}

void empty(int x) {
    queries++;
    x--;
    arr[x] = 0;
}

void fill(int x) {
    queries++;
    x--;
    arr[x] = cap[x];
}

int pour(int x, int y) {
    queries++;
    x--;
    y--;
    int ans = min(cap[y] - arr[y], arr[x]);
    arr[x] = 0;
    arr[y] += ans;
    return ans;
}

void answer(const vector<int>& ans) {
    dbg(queries);
    for (int i = 0; i < n; i++) {
        assert(ans[i] == cap[i]);
    }
}

}  // namespace grader

constexpr int maxn = 4e4 + 5;

void empty(int x) {
    x++;
    cout << "E " << x << endl;
#ifdef LOCAL
    grader::empty(x);
#endif
}

void fill(int x) {
    x++;
    cout << "F " << x << endl;
#ifdef LOCAL
    grader::fill(x);
#endif
}

int pour(int x, int y) {
    x++;
    y++;
    cout << "D " << x << " " << y << endl;
#ifdef LOCAL
    return grader::pour(x, y);
#else
    int ans;
    cin >> ans;
    return ans;
#endif
}

void answer(const vector<int>& arr) {
    cout << "A";
    for (auto& a : arr) {
        cout << " " << a;
    }
    cout << endl;
#ifdef LOCAL
    grader::answer(arr);
#endif
}

void solve() {
    int n, m;
    cin >> n >> m;
#ifdef LOCAL
    grader::init(n);
#endif
    vector<int> arr(n);
    int cmx = -1;
    auto arr_upd = [&](int ind) -> void {
        if (cmx == -1 || arr[ind] > cmx) {
            cmx = ind;
        }
    };
    empty(0);
    int cfilled = -1;
    for (int i = 0; i < n;) {
        for (; i < n && arr[i]; i++)
            ;
        if (i == n) {
            break;
        }
        int ci = i++;
        while (true) {
            for (; i < n && arr[i]; i++)
                ;
            int x1 = -1;
            if (i < n) {
                x1 = pour(i, ci);
                arr[ci] += x1;
                if (!x1) {
                    break;
                }
            }
            if (arr[ci] == m) {
                break;
            }
            int x2;
            if (cfilled != -1) {
                x2 = pour(cfilled, ci);
                cfilled = -1;
            } else {
                int cur = cmx;
                if (cur == -1) {
                    assert(!ci);
                    cur = 1;
                }
                fill(cur);
                x2 = pour(cur, ci);
            }
            arr[ci] += x2;
            if (!x2 || arr[ci] == m) {
                break;
            } else if (x1 != -1) {
                arr[i] = x1;
                arr_upd(i);
            }
        }
        arr_upd(ci);
        cfilled = ci;
    }
    answer(arr);
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}