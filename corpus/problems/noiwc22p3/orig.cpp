// corpus-written reference for WC 2022 P3: greedy expected-guesses search over the whole
// dictionary, the candidate set narrowed by the first letter and every mark so far; each
// decision is memoised by the game's history, so a letter's opening is searched once.
#include <bits/stdc++.h>
using namespace std;

static vector<array<char, 5>> dict;
static int num_words;

static int marks(const array<char, 5>& g, const array<char, 5>& a) {
	bool gold[5];
	for (int k = 0; k < 5; k++) gold[k] = g[k] == a[k];
	int p = 0;
	for (int k = 0; k < 5; k++) {
		int m = 0;
		if (gold[k]) m = 2;
		else
			for (int l = 0; l < 5; l++)
				if (!gold[l] && g[k] == a[l]) m = 1;
		p = p * 3 + m;
	}
	return p;
}

static double expect(int s) { return s == 1 ? 1.0 : s == 2 ? 1.5 : 1.0 + 0.46 * log2((double)s); }

static int choose(const vector<int>& cand, int remaining) {
	if (cand.size() <= 2 || remaining == 1) return cand[0];
	vector<char> is_cand(num_words);
	for (int c : cand) is_cand[c] = 1;
	double best = 1e18;
	int pick = cand[0];
	int cnt[243];
	for (int g = 0; g < num_words; g++) {
		memset(cnt, 0, sizeof cnt);
		for (int c : cand) cnt[marks(dict[g], dict[c])]++;
		double cost = 0;
		for (int p = 0; p < 242; p++)
			if (cnt[p]) cost += cnt[p] * expect(cnt[p]);
		if (cost < best - 1e-9 || (fabs(cost - best) <= 1e-9 && is_cand[g] && !is_cand[pick]))
			best = cost, pick = g;
	}
	return pick;
}

static map<string, int> memo;
static string history;
static vector<int> cand;
static int last;

extern "C" const char* guess(int, int remaining_guesses, char initial_letter, bool* gold, bool* silver) {
	if (remaining_guesses == 5) {
		history.assign(1, initial_letter);
		cand.clear();
		for (int i = 0; i < num_words; i++)
			if (dict[i][0] == initial_letter) cand.push_back(i);
	} else {
		int p = 0;
		for (int k = 0; k < 5; k++) p = p * 3 + (gold[k] ? 2 : silver[k] ? 1 : 0);
		vector<int> next;
		for (int c : cand)
			if (marks(dict[last], dict[c]) == p) next.push_back(c);
		cand.swap(next);
		history += char('0' + p / 27 % 9), history += char('0' + p % 27), history += char(last % 251);
		if (cand.empty()) return "paper";
	}
	auto it = memo.find(history);
	last = it != memo.end() ? it->second : memo[history] = choose(cand, remaining_guesses);
	static char word[6];
	copy(dict[last].begin(), dict[last].end(), word);
	return word;
}

extern "C" void init(int num_scramble, const char* scramble) {
	num_words = num_scramble;
	dict.resize(num_scramble);
	for (int i = 0; i < num_scramble; i++) copy(scramble + 5 * i, scramble + 5 * i + 5, dict[i].begin());
}

// stdin adaptation of the grader (corpus): read the dictionary and T, then play each game
// through guess() with the interactor's g / s / - marks.
int main() {
	int num;
	static char pure[8869 * 5 + 8];
	if (scanf("%d %44345s", &num, pure) != 2) return 0;
	init(num, pure);
	int t;
	scanf("%d", &t);
	for (int i = 1; i <= t; i++) {
		char letter[4];
		scanf("%3s", letter);
		bool gold[5] {}, silver[5] {};
		for (int rem = 5; rem >= 1; rem--) {
			const char* g = guess(i, rem, letter[0], gold, silver);
			printf("%.5s\n", g);
			fflush(stdout);
			char m[8];
			scanf("%7s", m);
			if (!strcmp(m, "ggggg")) break;
			for (int k = 0; k < 5; k++) gold[k] = m[k] == 'g', silver[k] = m[k] == 's';
		}
	}
}
