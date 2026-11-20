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
    for (int i = 0; i < n; i++) {
        empty(i);
    }
    int arr[n] {};
    while (true) {
        fill(1);
        int x = pour(1, 0);
        if (!x) {
            break;
        }
        arr[0] += x;
    }
    int pi = 0;
    bool filled[n] {};
    fill(filled + 2, filled + n, true);
    int fi = 0;
    for (int i = 1; i < n; i++) {
        if (filled[i]) {
            empty(i);
        }
        filled[i] = false;
        int x = pour(pi, i);
        if (x < arr[pi]) {
            arr[i] = x;
            fill(pi);
            continue;
        }
        while (x) {
            arr[i] += x;
            for (; fi < n && !filled[fi]; fi++);
            if (fi < n) {
                x = pour(fi, i);
                filled[fi] = false;
            } else {
                fill(pi);
                x = pour(pi, i);
            }
        }
        pi = i;
    }
    answer(vector<int>(arr, arr + n));
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}