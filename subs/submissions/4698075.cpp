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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

struct Psum {
    vector<int> p;

    Psum() : p {} {}
    Psum(const vector<int>& arr) : p(sz(arr) + 1) {
        partial_sum(begin(arr), end(arr), p.begin() + 1);
    }

    int query(int l, int r) const {
        return p[r + 1] - p[l];
    }
} arr[3][3];

const string conv = "ACT";

void init(string a, string b) {
    int n = sz(a);
    vector<int> cnts[3][3];
    for (auto& a : cnts) {
        for (auto& b : a) {
            b.resize(n);
        }
    }
    for (int i = 0; i < n; i++) {
        cnts[conv.find(a[i])][conv.find(b[i])][i]++;
    }
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            arr[i][j] = Psum(cnts[i][j]);
        }
    }
}

int get_distance(int l, int r) {
    int cnt[3][3];
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            cnt[i][j] = arr[i][j].query(l, r);
        }
    }
    int ans = 0;
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            if (i != j) {
                int x = min(cnt[i][j], cnt[j][i]);
                ans += x;
                cnt[i][j] -= x;
                cnt[j][i] -= x;
            }
        }
    }
    for (int i : {1, 2}) {
        int j = 1 ^ 2 ^ i;
        int &a = cnt[0][i], &b = cnt[i][j], &c = cnt[j][0];
        int x = min({a, b, c});
        a -= x;
        b -= x;
        c -= x;
        ans += 2 * x;
    }
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            if (i != j && cnt[i][j]) {
                return -1;
            }
        }
    }
    return ans;
}