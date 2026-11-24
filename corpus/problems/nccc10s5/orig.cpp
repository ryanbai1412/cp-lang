// nccc10s5 reference. Every empty cell not on the escalators' walkable component may as well
// be a shop, so the hallways plus escalators form one connected set H holding both
// escalators, and the answer is the most shop-hallway edges over such H. Broken-profile DP
// over the narrow side (at most 9 since R * C <= 99): each frontier cell is a pillar, a shop,
// or an H cell with its component label and whether it is an escalator.
#include <bits/stdc++.h>
using namespace std;

int R, C;
vector<string> g;
using State = uint64_t;
const int B = 5;

int slot(State s, int i) { return (s >> (B * i)) & 31; }
State put(State s, int i, int v) { return (s & ~(State(31) << (B * i))) | (State(v) << (B * i)); }
bool closedOf(State s) { return s >> 63; }

State normalize(State s) {
    int map_[16] = {}, next = 1;
    State out = s & (State(1) << 63);
    for (int i = 0; i < C; i++) {
        int v = slot(s, i);
        if (v >= 2) {
            int l = v >> 1;
            if (!map_[l]) map_[l] = next++;
            v = map_[l] << 1 | (v & 1);
        }
        out = put(out, i, v);
    }
    return out;
}

int main() {
    cin >> R >> C;
    g.resize(R);
    for (auto& r : g) cin >> r;
    if (C > R) {
        vector<string> t(C, string(R, ' '));
        for (int i = 0; i < R; i++)
            for (int j = 0; j < C; j++) t[j][i] = g[i][j];
        g = t;
        swap(R, C);
    }
    int lastEsc = -1;
    for (int i = 0; i < R; i++)
        for (int j = 0; j < C; j++)
            if (g[i][j] == 'U' || g[i][j] == 'D') lastEsc = i * C + j;
    unordered_map<State, int> cur{{0, 0}}, nxt;
    for (int r = 0; r < R; r++)
        for (int c = 0; c < C; c++) {
            nxt.clear();
            char ch = g[r][c];
            int idx = r * C + c;
            for (auto [s, val] : cur) {
                int up = slot(s, c), left = c ? slot(s, c - 1) : 0;
                bool closed = closedOf(s);
                auto finish = [&](State t, int gain) {
                    if (up >= 2) {
                        bool alive = false, other = false;
                        for (int i = 0; i < C; i++) {
                            int v = slot(t, i);
                            if (v >= 2 && (v >> 1) == (up >> 1)) alive = true;
                            else if (v >= 2) other = true;
                        }
                        if (!alive) {
                            if (other || idx < lastEsc) return;
                            t |= State(1) << 63;
                        }
                    }
                    t = normalize(t);
                    auto it = nxt.find(t);
                    if (it == nxt.end() || it->second < val + gain) nxt[t] = val + gain;
                };
                if (ch == '#') {
                    finish(put(s, c, 0), 0);
                    continue;
                }
                if (ch == '.') {
                    int gain = (up >= 2 && !(up & 1)) + (left >= 2 && !(left & 1));
                    finish(put(s, c, 1), gain);
                }
                if (closed) continue;
                bool esc = ch != '.';
                int gain = esc ? 0 : (up == 1) + (left == 1);
                State t = s;
                int label = 15;
                if (left >= 2) {
                    int l = left >> 1;
                    for (int i = 0; i < C; i++)
                        if (slot(t, i) >= 2 && (slot(t, i) >> 1) == l) t = put(t, i, label << 1 | (slot(t, i) & 1));
                }
                if (up >= 2) {
                    int l = up >> 1;
                    for (int i = 0; i < C; i++)
                        if (slot(t, i) >= 2 && (slot(t, i) >> 1) == l) t = put(t, i, label << 1 | (slot(t, i) & 1));
                }
                t = put(t, c, label << 1 | esc);
                if (up >= 2) up = 15 << 1;
                finish(t, gain);
            }
            swap(cur, nxt);
        }
    int best = 0;
    for (auto [s, val] : cur) {
        set<int> labels;
        for (int i = 0; i < C; i++)
            if (slot(s, i) >= 2) labels.insert(slot(s, i) >> 1);
        if (labels.size() <= 1 && (closedOf(s) || labels.size() == 1)) best = max(best, val);
    }
    cout << best << '\n';
}
