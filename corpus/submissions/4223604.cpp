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

mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
int n;
bool vis[maxn];
pair<int, vector<int>> opt;
vector<int> cur;

void dfs(int u, int cw = 0) {
	if (sz(cur) == n - 1) {
		cur.push_back(u);
		opt = max(opt, pair<int, vector<int>> {cw, cur});
		cur.pop_back();
		return;
	}
	vis[u] = true;
	cur.push_back(u);
	int copt = 0;
	vector<int> ovs;
	for (int i = 0; i < n; i++) {
		if (!vis[i]) {
			int w = abs(u - i);
			if (!prime[w]) {
				w = 0;
			}
			if (w > copt) {
				copt = w;
				ovs.clear();
				ovs.push_back(i);
			} else if (w == copt) {
				ovs.push_back(i);
			}
		}
	}
	assert(sz(ovs));
	int nxt = ovs[cowng() % sz(ovs)];
	dfs(nxt, cw + copt);
	vis[u] = false;
	cur.pop_back();
}

void solve() {
	cin >> n;
	auto now = clock();
	while (double(clock() - now) < 1.9 * CLOCKS_PER_SEC) {
		dfs(cowng() % n);
	}
	dbg(opt.first);
	auto cur = opt.second;
	for (int i = 0; i < n; i++) {
		cout << cur[i] + 1 << " \n"[i == n - 1];
	}
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