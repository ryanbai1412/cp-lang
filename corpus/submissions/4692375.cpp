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

int rectangle(int n, int m, int qn, int qm, int arr[3001][3001]) {
    vector<int> aarr;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            aarr.push_back(arr[i][j]);
        }
    }
    sort(begin(aarr), end(aarr));
    auto check = [&](int x) -> bool {
        int psums[n + 1][m + 1] {};
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < m; j++) {
                psums[i + 1][j + 1] = psums[i + 1][j] + psums[i][j + 1] -
                                      psums[i][j] + (arr[i][j] <= aarr[x]);
            }
        }
        int targ = qn * qm / 2;
        for (int i = 0; i + qn <= n; i++) {
            for (int j = 0; j + qm <= m; j++) {
                int x1 = i, x2 = i + qn, y1 = j, y2 = j + qm;
                if (psums[x2][y2] - psums[x1][y2] - psums[x2][y1] +
                        psums[x1][y1] >
                    targ) {
                    if (aarr[x] == 8) {
                        dbg(i, j);
                    }
                    return true;
                }
            }
        }
        return false;
    };
    int l = 0, r = sz(aarr) - 1;
    while (l < r) {
        int mid = (l + r) / 2;
        if (check(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    return aarr[l];
}