// IOI 2015 Teams: persistent segment tree over B, versioned by A, and the
// dp[i] = min_j dp[j] + #{K_j < A <= K_i, B >= K_i} - K_i check with a stack of
// candidates (an older candidate overtakes a newer one once K passes their crossover).
#include <bits/stdc++.h>
using namespace std;

static int n;
static vector<int> root, lc, rc, cntv;

static int add(int prev, int l, int r, int pos) {
    int cur = cntv.size();
    lc.push_back(lc[prev]); rc.push_back(rc[prev]); cntv.push_back(cntv[prev] + 1);
    if (l == r) return cur;
    int m = (l + r) / 2;
    if (pos <= m) { int c = add(lc[prev], l, m, pos); lc[cur] = c; }
    else { int c = add(rc[prev], m + 1, r, pos); rc[cur] = c; }
    return cur;
}

// students in version hi minus version lo with B >= y
static int count_ge(int hi, int lo, int l, int r, int y) {
    if (r < y || cntv[hi] - cntv[lo] == 0) return 0;
    if (l >= y) return cntv[hi] - cntv[lo];
    int m = (l + r) / 2;
    return count_ge(lc[hi], lc[lo], l, m, y) + count_ge(rc[hi], rc[lo], m + 1, r, y);
}

static int between(int a1, int a2, int y) {  // K-values a1 < A <= a2, B >= y
    if (y > n) return 0;
    return count_ge(root[a2], root[a1], 1, n, y);
}

void init(int N, int A[], int B[]) {
    n = N;
    lc.assign(1, 0); rc.assign(1, 0); cntv.assign(1, 0);
    vector<vector<int>> byA(n + 1);
    for (int i = 0; i < n; i++) byA[A[i]].push_back(B[i]);
    root.assign(n + 1, 0);
    for (int a = 1; a <= n; a++) {
        int r = root[a - 1];
        for (int b : byA[a]) r = add(r, 1, n, b);
        root[a] = r;
    }
}

int can(int M, int K[]) {
    long long sum = 0;
    for (int i = 0; i < M; i++) sum += K[i];
    if (sum > n) return 0;
    vector<int> k(K, K + M);
    sort(k.begin(), k.end());
    k.insert(k.begin(), 0);
    vector<long long> dp(M + 1, 0);
    // crossover: smallest y >= k[b] from which candidate a is at least as good as b
    auto cross = [&](int a, int b) -> long long {
        int lo = k[b], hi = n + 1;
        if (dp[a] + between(k[a], k[b], hi) > dp[b]) return LLONG_MAX;
        while (lo < hi) {
            int mid = (lo + hi) / 2;
            if (dp[a] + between(k[a], k[b], mid) <= dp[b]) hi = mid; else lo = mid + 1;
        }
        return lo;
    };
    vector<int> st{0};
    vector<long long> at{LLONG_MIN};  // at[t]: from which K st[t-1] beats st[t]
    for (int i = 1; i <= M; i++) {
        while (st.size() > 1 && at.back() <= k[i]) { st.pop_back(); at.pop_back(); }
        int j = st.back();
        dp[i] = dp[j] + between(k[j], k[i], k[i]) - k[i];
        if (dp[i] < 0) return 0;
        while (true) {
            long long t = cross(st.back(), i);
            if (st.size() > 1 && t <= at.back()) {
                // st.back() is overtaken by its predecessor no later than i overtakes... drop it
                st.pop_back(); at.pop_back();
                continue;
            }
            st.push_back(i); at.push_back(t);
            break;
        }
    }
    return 1;
}

int main() {
    int N;
    if (scanf("%d", &N) != 1) return 1;
    vector<int> A(N), B(N);
    for (int i = 0; i < N; i++) scanf("%d %d", &A[i], &B[i]);
    init(N, A.data(), B.data());
    int Q;
    scanf("%d", &Q);
    vector<int> K;
    while (Q--) {
        int M;
        scanf("%d", &M);
        K.resize(M);
        for (auto &x : K) scanf("%d", &x);
        printf("%d\n", can(M, K.data()));
    }
}
