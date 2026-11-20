#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

bool valid(vector<int> arr, int k) {
	if(arr[0] == 0) {
		return true;
	}
	int n = sz(arr);
	int tot = 0;
	for(int j = 0; j < n; j++) {
		int x = 1;
		for(int a = 0; a < k; a++) {
			x *= arr[(j + a) % n];
		}
		tot += x;
	}
	return tot == 0;
}

mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());

vector<int> solve(int n, int k) {
	if(n & 1) {
		return {0};
	}else if(n == k) {
		return {0};
	}else if(n % 4 == 2 && (k & 1) == 0) {
		return {0};
	}
	if(k & 1) {
		vector<int> ans(n, 1);
		for(int i = 0; i < n; i++) {
			if(i & 1) {
				ans[i] = -1;
			}
		}
		return ans;
	}
	int kx = __builtin_ctz(k);
	if(__builtin_ctz(n) > kx) {
		vector<int> ans(n, 1);
		kx = 1 << (kx + 1);
		for(int i = 0; i < n; i += kx) {
			ans[i] = -1;
		}
		return ans;
	}
	vector<int> ans(n);
	int tries = 0;
	do {
		generate(begin(ans), end(ans), []() {return 2 * (cowng() & 1) - 1;});
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
	assert(valid(ans, k));
	for(int i = 0; i < sz(ans); i++) {
		if(i) {
			cout << " ";
		}
		cout << ans[i];
	}
	cout << endl;
}