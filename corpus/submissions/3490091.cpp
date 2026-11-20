#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

//https://martin.ankerl.com/2018/12/08/fast-random-bool/
template <typename U = uint64_t> struct fast_rng {
	static constexpr U s_mask = sizeof(U) * 8 - 1;
	mt19937_64 rng = mt19937_64(chrono::steady_clock::now().time_since_epoch().count());
	U m_rand = 0;
	uint_fast8_t m_counter = 0;

	bool operator()() {
		auto b = m_counter & s_mask;
		if(__builtin_expect(b == 0, 0)) {
			m_rand = rng();
		}
		++m_counter;
		return (m_rand >> b) & 1;
	}
};

bool valid(vector<bool> arr, int k) {
	if(arr[0] == 0) {
		return true;
	}
	int n = sz(arr);
	bool cur = 0;
	int ans = 0, j = 0;
	for(int i = 0; i < n; i++) {
		while(j < i + k) {
			cur ^= arr[j % n];
			j++;
		}
		ans += (cur << 1) - 1;
		cur ^= arr[i];
	}
	return ans == 0;
}

vector<bool> solve(int n, int k) {
	if(n & 1) {
		return {0};
	}else if(n == k) {
		return {0};
	}else if(n % 4 == 2 && (k & 1) == 0) {
		return {0};
	}
	if(k & 1) {
		vector<bool> ans(n, 1);
		for(int i = 0; i < n; i += 2) {
			ans[i] = 0;
		}
		return ans;
	}
	int nx = __builtin_ctz(n), kx = __builtin_ctz(k);
	if(nx > kx) {
		vector<bool> ans(n, 1);
		kx = 1 << (kx + 1);
		for(int i = 0; i < n; i += kx) {
			ans[i] = 0;
		}
		return ans;
	}
	fast_rng cowng;
	vector<bool> ans(n);
	int tries = 0;
	do {
		for(int i = 0; i < n; i++) {
			ans[i] = cowng();
		}
		tries++;
	}while(!valid(ans, k));
	cerr << tries << endl;
	return ans;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, k;
	cin >> n >> k;
	auto ans = solve(n, k);
	if(sz(ans)==1) {
		cout << "0" << endl;
	}else {
		for(int i = 0; i < sz(ans); i++) {
			if(i) {
				cout << " ";
			}
			cout << ans[i] * 2 - 1;
		}
		cout << endl;
	}
//	mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
//	while(true) {
//		int n = cowng() % 1000000 + 1, k = cowng() % n + 1;
//		cerr << n << " " << k << endl;
//		solve(n, k);
//	}
}