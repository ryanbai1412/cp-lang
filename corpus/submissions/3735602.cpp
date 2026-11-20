#include "bits/extc++.h"

using namespace std;

#define long uint64_t
#define sz(x) int((x).size())

struct Int {
	bool o = false;
	long x = 0;
};

Int operator+(Int a, Int b) {
	static long mask = long(1) << 63;
	return {a.o || b.o || (!!(a.x & mask) && !!(b.x & mask)), a.x + b.x};
}

bool operator>(Int a, long b) {
	if(a.o) {
		return false;
	}
	return a.x > b;
}

vector<int> sent;

#ifdef LOCAL

void send(int x) {
	sent.push_back(x);
}

void output(int x) {
	cout << x << endl;
}

#endif

const int maxn = 40;

Int dp[maxn + 1][32];

void pcomp() {
	fill(dp[0], dp[0] + 32, Int{false, 1});
	for(int i = 1; i <= maxn; i++) {
		for(int j = 0; j < 32; j++) {
			for(int k = j; k < 32; k++) {
				dp[i][j] = dp[i][j] + dp[i - 1][k];
			}
		}
	}
}

vector<int> ans;

void gen(int n, long x) {
	if(n) {
		int prev = sz(ans) ? ans.back() : 0;
		for(int j = prev; j < 32; j++) {
			if(dp[n - 1][j] > x) {
				ans.push_back(j);
				gen(n - 1, x);
				return;
			}
			x = x - dp[n - 1][j].x;
		}
	}
}

vector<int> gen(long x) {
	ans.clear();
	gen(maxn, x);
	return ans;
}

long from(const vector<int> &arr) {
	assert(sz(arr) == maxn);
	long ans = 0;
	for(int i = 0; i < maxn; i++) {
		for(int j = i ? arr[i - 1] : 0; j < arr[i]; j++) {
			ans = ans + dp[maxn - i - 1][j].x;
		}
	}
	return ans;
}

void encode(int n, int arr[]) {
	if(!dp[0][0].x) {
		pcomp();
	}
	for(int i = 0; i < n; i += 8) {
		long chash = 0;
		for(int j = i; j < min(n, i + 8); j++) {
			if(j < n) {
				chash |= long(arr[j]) << ((j - i) << 3);
			}
		}
		auto cur = gen(chash);
		int iters = 5 * (min(n, i + 8) - i);
		for(int j = maxn - iters; j < maxn; j++) {
			send(((i >> 3) << 5) | cur[j]);
		}
	}
}

void decode(int n, int m, int arr[]) {
	if(!dp[0][0].x) {
		pcomp();
	}
	vector<int> message[n];
	for(int i = 0; i < m; i++) {
		message[arr[i] >> 5].push_back(arr[i] & 31);
	}
	for(int i = 0; i < n; i += 8) {
		auto &cm = message[i / 8];
		while(sz(cm) < maxn) {
			cm.insert(cm.begin(), 0);
		}
		sort(begin(cm), end(cm));
		long cur = from(cm);
		for(int j = i; j < min(n, i + 8); j++) {
			output(cur & 255);
			cur >>= 8;
		}
	}
}