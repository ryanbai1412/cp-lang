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

const long mod = 1e9 + 7;

long bpow(long base, long exp) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

long inv(long x) {
	return bpow(x, mod - 2);
}

const int maxn = 4005;

bool from[maxn];
int arr[maxn], graph[maxn][2], siz[maxn], sum[maxn], can[maxn];
long fact[maxn], ifact[maxn], memo[maxn][maxn];

void pcomp() {
	fact[0] = ifact[0] = 1;
	for (int i = 1; i < maxn; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
		ifact[i] = inv(fact[i]);
	}
}

long choose(int n, int k) {
	return fact[n] * ifact[k] % mod * ifact[n - k] % mod;
}

long pick(int n, int k) {
	return fact[n] * ifact[n - k] % mod;
}

long ways(int a, int b) {
	return pick(a + b, b);
}

void pdfs(int u) {
	siz[u] = 1;
	sum[u] = arr[u];
	auto go = [&](int v, bool b) -> void {
		if (v == -1) {
			return;
		}
		from[v] = b;
		pdfs(v);
		siz[u] += siz[v];
		sum[u] += sum[v];
	};
	go(graph[u][0], false);
	go(graph[u][1], true);
	can[u] = siz[u] - sum[u];
}

long dp(int u, int k) {
	if (k > can[u]) {
		return 0;
	}
	int ccnt = (graph[u][0] != -1) + (graph[u][1] != -1);
	if (!ccnt) {
		return 1;
	}
	long& ans = memo[u][k];
	if (ans != -1) {
		return ans;
	}
	if (ccnt == 1) {
		long x;
		int v = graph[u][0] ^ graph[u][1] ^ -1;
		if (k == can[u]) {
			x = dp(v, k + arr[u] - 1);
		} else {
			x = dp(v, k + arr[u]);
		}
		ans = ways(k, arr[u]) * x % mod;
	} else if (ccnt == 2) {
		auto [l, r] = graph[u];
		if (from[u]) {
			swap(l, r);
		}
		ans = 0;
		for (int i = 0; i <= arr[u]; i++) {
			int bl = i, br = arr[u] - i;
			int al = min(k, can[l] - bl), ar = k - al;
			if (al >= 0) {
				int xr = ar + br;
				if (al + bl == can[l] && xr > can[r]) {
					xr--;
				}
				ans = (ans + choose(arr[u], i) * ways(al, bl) % mod * ways(ar, br) % mod *
								 dp(l, al + bl) % mod * dp(r, xr)) %
					  mod;
			}
			dbg(i, ans);
		}
	} else {
		assert(false);
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < m; i++) {
		int u;
		cin >> u;
		u--;
		arr[u]++;
	}
	for (int i = 0; i < n; i++) {
		cin >> graph[i][0] >> graph[i][1];
		graph[i][0]--;
		graph[i][1]--;
	}
	pdfs(0);
	if (*min_element(begin(can), end(can)) < 0) {
		cout << 0 << endl;
		return;
	}
	memset(memo, -1, sizeof(memo));
	cout << dp(0, 0) << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	pcomp();
	solve();
}