// The original is a grader-style submission (label and find_next_station,
// no main). This reads the port's input format, labels the tree, routes
// each query one hop by labels alone, and prints the station reached.
#include "../../../../corpus/submissions/4725602.cpp"

int main() {
    int n;
    scanf("%d", &n);
    vector<int> us(n - 1), vs(n - 1);
    vector<vector<int>> near(n);
    for (int i = 0; i < n - 1; i++) {
        scanf("%d %d", &us[i], &vs[i]);
        near[us[i]].push_back(vs[i]);
        near[vs[i]].push_back(us[i]);
    }
    vector<int> labels = label(n, n - 1, us, vs), station(n);
    for (int i = 0; i < n; i++) station[labels[i]] = i;
    int q;
    scanf("%d", &q);
    while (q--) {
        int s, t;
        scanf("%d %d", &s, &t);
        vector<int> c;
        for (int v : near[s]) c.push_back(labels[v]);
        sort(c.begin(), c.end());
        printf("%d\n", station[find_next_station(labels[s], labels[t], c)]);
    }
}
