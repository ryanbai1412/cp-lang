//! A fact table keyed by file span.

use cpl_diag::Span;

/// Values by file span, sorted by start then end and found by binary search.
/// Every per-span table of the facts has this one shape, so the wasm build
/// carries one sort for all of them instead of a hash table per value type.
#[derive(Clone, Debug)]
pub(crate) struct BySpan<T>(Vec<(Span, T)>);

impl<T> Default for BySpan<T> {
    fn default() -> Self {
        BySpan(Vec::new())
    }
}

impl<T> BySpan<T> {
    /// `entries` in the order found; of several at one span, the last wins.
    pub(crate) fn new(entries: Vec<(Span, T)>) -> Self {
        Self::merged(entries, |_, _| {})
    }

    /// Like [`BySpan::new`], but `prefer(kept, earlier)` may swap in an
    /// earlier entry's value at the same span.
    pub(crate) fn merged(mut entries: Vec<(Span, T)>, prefer: impl Fn(&mut T, &mut T)) -> Self {
        let mut at = order(entries.iter().map(|(s, _)| *s).collect());
        for k in 0..at.len() {
            let mut cur = k;
            while at[cur] as usize != k {
                let next = at[cur] as usize;
                entries.swap(cur, next);
                at[cur] = cur as u32;
                cur = next;
            }
            at[cur] = cur as u32;
        }
        entries.dedup_by(|earlier, kept| {
            let same = earlier.0 == kept.0;
            if same {
                prefer(&mut kept.1, &mut earlier.1);
            }
            same
        });
        BySpan(entries)
    }

    /// The value at exactly `span`.
    pub(crate) fn get(&self, span: Span) -> Option<&T> {
        (self.0)
            .binary_search_by_key(&(span.start, span.end), |(s, _)| (s.start, s.end))
            .ok()
            .map(|at| &self.0[at].1)
    }

    /// Every entry inside `span`, by start, then end.
    pub(crate) fn within(&self, span: Span) -> impl Iterator<Item = (Span, &T)> {
        let from = (self.0).partition_point(|(s, _)| s.start < span.start);
        self.0[from..]
            .iter()
            .take_while(move |(s, _)| s.start <= span.end)
            .filter(move |(s, _)| s.end <= span.end)
            .map(|(s, v)| (*s, v))
    }
}

/// Where each of `spans` goes, by start, then end, then latest first: the
/// one sort every table shares.
fn order(spans: Vec<Span>) -> Vec<u32> {
    let mut keys: Vec<(u32, u32, u32)> = (spans.iter().enumerate())
        .map(|(i, s)| (s.start, s.end, u32::MAX - i as u32))
        .collect();
    keys.sort_unstable();
    keys.into_iter().map(|(_, _, i)| u32::MAX - i).collect()
}

#[cfg(test)]
mod tests {
    use super::BySpan;
    use cpl_diag::Span;

    #[test]
    fn the_last_entry_at_a_span_wins_and_lookups_are_in_source_order() {
        let s = Span::new;
        let table = BySpan::new(vec![
            (s(4, 6), 'a'),
            (s(0, 9), 'b'),
            (s(4, 6), 'c'),
            (s(0, 2), 'd'),
        ]);
        assert_eq!(table.get(s(4, 6)), Some(&'c'));
        assert_eq!(table.get(s(4, 5)), None);
        let inside: Vec<_> = table.within(s(0, 9)).map(|(_, v)| *v).collect();
        assert_eq!(inside, ['d', 'b', 'c']);
        let vague = BySpan::merged(vec![(s(1, 2), 1), (s(1, 2), 0)], |kept, earlier| {
            if *kept == 0 {
                std::mem::swap(kept, earlier)
            }
        });
        assert_eq!(vague.get(s(1, 2)), Some(&1));
    }
}
