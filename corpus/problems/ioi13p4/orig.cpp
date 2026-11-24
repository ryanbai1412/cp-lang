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

extern "C" {
int tryCombination(int S[]);
void answer(int S[], int D[]);
void exploreCave(int N);
}

int tc(int arr[]) {
    int ans = tryCombination(arr);
    if (ans == -1) {
        return 1e9;
    }
    return ans;
}

void exploreCave(int n) {
    int pos[n], door[n], swtch[n];
    bool vis[n] {};
    for (int i = 0; i < n; i++) {
        int cur[n] {};
        for (int j = 0; j < i; j++) {
            cur[door[j]] = pos[door[j]];
        }
        int ty;
        if (tc(cur) > i) {
            ty = 0;
        } else {
            ty = 1;
        }
        vector<int> poss;
        for (int j = 0; j < n; j++) {
            if (!vis[j]) {
                poss.push_back(j);
            }
        }
        auto check = [&](int x) -> bool {
            fill(cur, cur + n, !ty);
            for (int j = 0; j < i; j++) {
                cur[door[j]] = pos[door[j]];
            }
            for (int j = 0; j <= x; j++) {
                cur[poss[j]] = ty;
            }
            return tc(cur) > i;
        };
        int l = 0, r = sz(poss) - 1;
        while (l < r) {
            int mid = (l + r) / 2;
            if (check(mid)) {
                r = mid;
            } else {
                l = mid + 1;
            }
        }
        pos[door[i] = poss[l]] = ty;
        vis[door[i]] = true;
        swtch[door[i]] = i;
        dbg(check(0), door[i], ty);
    }
    answer(pos, swtch);
}
// The original is the function the IOI grader calls. This `main` stands in for
// the grader over interactor.py's protocol: read N; `try s..` gets the first
// closed door (or -1); `answer S.. D..` ends the run.
static int n_in;
extern "C" int tryCombination(int S[]) {
    printf("try");
    for (int w = 0; w < n_in; w++) printf(" %d", S[w]);
    printf("\n");
    fflush(stdout);
    int first;
    if (scanf("%d", &first) != 1) exit(0);
    return first;
}
extern "C" void answer(int S[], int D[]) {
    printf("answer");
    for (int w = 0; w < n_in; w++) printf(" %d", S[w]);
    for (int w = 0; w < n_in; w++) printf(" %d", D[w]);
    printf("\n");
    fflush(stdout);
    exit(0);
}
int main() {
    if (scanf("%d", &n_in) != 1) return 1;
    exploreCave(n_in);
}
