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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 2e6;

bool prime[maxn + 1];
vector<int> primes;

void pcomp() {
	memset(prime, 1, sizeof(prime));
	prime[0] = prime[1] = false;
	for (int i = 2; i <= maxn; i++) {
		if (prime[i]) {
			primes.push_back(i);
			if (long(i) * i <= maxn) {
				for (int j = i * i; j <= maxn; j += i) {
					prime[j] = false;
				}
			}
		}
	}
}

int n;
vector<int> stk;
bool vis[maxn + 1];

void dfs(int ind) {
	if (ind == n + 1) {
		for (int i = 0; i < n; i++) {
			cout << stk[i] << " \n"[i == n - 1];
		}
		exit(0);
	}
	for (auto& a : primes) {
		int cur = a - ind;
		if (1 <= cur && cur <= n && !vis[cur]) {
			vis[cur] = true;
			stk.push_back(cur);
			dfs(ind + 1);
			vis[cur] = false;
			stk.pop_back();
		}
	}
}

void solve() {
	cin >> n;
	dfs(1);
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}