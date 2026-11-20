#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

const int maxn = 5001;
int n;
int bit[maxn + 1];

void add(int ind) {
	ind++;
	while(ind <= n) {
		bit[ind]++;
		ind += ind & -ind;
	}
}

int psum(int ind) {
	int ret = 0;
	while(ind > 0) {
		ret += bit[ind];
		ind -= ind & -ind;
	}
	return ret;
}

int lsum(int ind) {
	return psum(n) - psum(ind);
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	cin >> n;
	int arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	int sz = 0;
	bool prev[n] {};
	for(int i = 0; i < n; i++) {
		for(int j = i - 1; j >= 0 && sz < arr[i]; j--) {
			if(!prev[j]) {
				prev[j] = true;
				sz++;
			}
		}
		memset(bit, 0, sizeof(bit));
		if(sz == arr[i]) {
			prev[i] = true;
			sz++;
		}
		long ans = 0;
		for(int j = 0; j <= i; j++) {
			ans += lsum(arr[j] + 1 - prev[j]);
			if(prev[j]) {
				add(arr[j]);
			}
		}
		if(i) {
			cout << " ";
		}
		cout << ans;
	}
	cout << endl;
}