// The original is a grader-style submission (find_split, no main). This
// reads the port's input format and prints the labels it returns.
#include "../../../../corpus/submissions/5661848.cpp"

int main() {
    int n, m, a, b;
    scanf("%d %d %d %d", &n, &m, &a, &b);
    int c = n - a - b;
    vector<int> us(m), vs(m);
    for (int i = 0; i < m; i++) scanf("%d %d", &us[i], &vs[i]);
    for (int x : find_split(n, a, b, c, us, vs)) printf("%d ", x);
    puts("");
}
