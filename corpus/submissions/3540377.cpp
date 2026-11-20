#include "bits/stdc++.h"

using namespace std;

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
	int n;
	cin >> n;
	vector<pair<int, int>> restraints;
	mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
	vector<int> arr(n);
	iota(begin(arr), end(arr), 1);
	while(true) {
		while(true) {
			shuffle(begin(arr), end(arr), cowng);
			for(auto &[a, b]: restraints) {
				if(arr[a] == b) {
					goto loop;
				}
			}
			break;
			loop:;
		}
		for(int i = 0; i < n; i++) {
			cout << arr[i] << " ";
		}
		cout << endl;
		cout.flush();
		int x;
		cin >> x;
		if(x == -1 || x == 0) {
			return 0;
		}else {
			restraints.emplace_back(x - 1, arr[x - 1]);
		}
	}
}