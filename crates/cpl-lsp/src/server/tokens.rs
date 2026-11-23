//! Semantic tokens, whole or as an edit to the tokens last sent.

use super::{semantic_tokens, uri_of, Reply, Server};
use cpl_json::{obj, Value};

impl Server {
    /// The answer to a semantic tokens request, or `None` for another method.
    pub(super) fn tokens(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Option<Result<Reply, (i32, String)>> {
        let delta = match method {
            "textDocument/semanticTokens/full" => false,
            "textDocument/semanticTokens/full/delta" => true,
            _ => return None,
        };
        Some(uri_of(params).map(|uri| {
            let uri = uri.to_string();
            let data = self
                .db
                .analyze(&uri)
                .map_or_else(Vec::new, |analysis| semantic_tokens(&analysis));
            self.token_ids += 1;
            let id = self.token_ids;
            let previous = params.get("previousResultId").and_then(Value::as_str);
            let sent = self.tokens.entry(uri).or_default();
            let old = previous
                .filter(|_| delta)
                .and_then(|previous| sent.iter().find(|(old, _)| old.to_string() == previous))
                .map(|(_, old)| old.clone());
            sent.push((id, data.clone()));
            if sent.len() > KEPT {
                sent.remove(0);
            }
            let numbers = |data: &[u32]| Value::Array(data.iter().map(|&n| n.into()).collect());
            let id = ("resultId", id.to_string().into());
            Reply::result(match old {
                Some(old) => {
                    let (start, deleted, inserted) = edit(&old, &data);
                    obj([
                        id,
                        (
                            "edits",
                            vec![obj([
                                ("start", start.into()),
                                ("deleteCount", deleted.into()),
                                ("data", numbers(inserted)),
                            ])]
                            .into(),
                        ),
                    ])
                }
                None => obj([id, ("data", numbers(&data))]),
            })
        }))
    }
}

/// How many of a document's last token sets a delta may start from.
const KEPT: usize = 4;

/// The one splice that turns `old` into `new`: where it starts, how many
/// numbers it deletes, and what it inserts.
pub(crate) fn edit<'a>(old: &[u32], new: &'a [u32]) -> (usize, usize, &'a [u32]) {
    let prefix = old.iter().zip(new).take_while(|(a, b)| a == b).count();
    let rest = old.len().min(new.len()) - prefix;
    let suffix = old
        .iter()
        .rev()
        .zip(new.iter().rev())
        .take(rest)
        .take_while(|(a, b)| a == b)
        .count();
    (
        prefix,
        old.len() - prefix - suffix,
        &new[prefix..new.len() - suffix],
    )
}
