//! The green/red tree (rust-analyzer's design).
//!
//! A green node is immutable and position-free: its kind, its width and
//! its children, shared behind an `Arc`, so an unchanged subtree is reused
//! as is. A red node is a cursor over a green one that knows its parent and
//! its offset in the file, made on demand while walking.

use crate::file::NodeKind;
use crate::token::TokenKind;
use cpl_diag::Span;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GreenToken(Arc<GreenTokenData>);

#[derive(PartialEq, Eq, Hash)]
struct GreenTokenData {
    kind: TokenKind,
    text: Box<str>,
}

impl GreenToken {
    pub fn new(kind: TokenKind, text: &str) -> GreenToken {
        GreenToken(Arc::new(GreenTokenData {
            kind,
            text: text.into(),
        }))
    }

    pub fn kind(&self) -> TokenKind {
        self.0.kind
    }

    pub fn text(&self) -> &str {
        &self.0.text
    }

    pub fn width(&self) -> u32 {
        self.0.text.len() as u32
    }
}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {:?}", self.kind(), self.text())
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GreenElement {
    Node(GreenNode),
    Token(GreenToken),
}

impl GreenElement {
    fn width(&self) -> u32 {
        match self {
            GreenElement::Node(n) => n.width(),
            GreenElement::Token(t) => t.width(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GreenNode(Arc<GreenNodeData>);

#[derive(PartialEq, Eq, Hash)]
struct GreenNodeData {
    kind: NodeKind,
    width: u32,
    children: Box<[GreenElement]>,
}

impl GreenNode {
    pub fn new(kind: NodeKind, children: Vec<GreenElement>) -> GreenNode {
        let width = children.iter().map(GreenElement::width).sum();
        GreenNode(Arc::new(GreenNodeData {
            kind,
            width,
            children: children.into(),
        }))
    }

    pub fn kind(&self) -> NodeKind {
        self.0.kind
    }

    pub fn width(&self) -> u32 {
        self.0.width
    }

    pub fn children(&self) -> &[GreenElement] {
        &self.0.children
    }
}

impl fmt::Debug for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(&format!("{:?}", self.kind()))
            .field(&self.children())
            .finish()
    }
}

/// Builds a green tree from a preorder stream of starts, tokens and
/// finishes. A checkpoint lets a parser wrap what it already built in a
/// node it only now knows it needs, as a binary expression wraps its
/// left operand.
#[derive(Default)]
pub struct GreenBuilder {
    parents: Vec<(NodeKind, usize)>,
    children: Vec<GreenElement>,
}

#[derive(Clone, Copy, Debug)]
pub struct Checkpoint(usize);

impl GreenBuilder {
    pub fn start_node(&mut self, kind: NodeKind) {
        self.parents.push((kind, self.children.len()));
    }

    pub fn token(&mut self, kind: TokenKind, text: &str) {
        self.children
            .push(GreenElement::Token(GreenToken::new(kind, text)));
    }

    pub fn finish_node(&mut self) {
        let Some((kind, first)) = self.parents.pop() else {
            debug_assert!(false, "finish_node without start_node");
            return;
        };
        let children = self.children.split_off(first);
        self.children
            .push(GreenElement::Node(GreenNode::new(kind, children)));
    }

    /// Adds a node built apart, as an item parsed on its own.
    pub fn node(&mut self, node: GreenNode) {
        self.children.push(GreenElement::Node(node));
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.children.len())
    }

    /// Starts a node whose first child is the one built at `checkpoint`.
    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: NodeKind) {
        debug_assert!(self.parents.last().is_none_or(|p| p.1 <= checkpoint.0));
        self.parents.push((kind, checkpoint.0));
    }

    /// Gives the node finished last another kind: a construct whose kind
    /// is known only once it is read, as `(a)` against `(a,)`.
    pub(crate) fn relabel_last(&mut self, kind: NodeKind) {
        if let Some(GreenElement::Node(node)) = self.children.last_mut() {
            *node = GreenNode::new(kind, node.children().to_vec());
        }
    }

    /// The finished root: the one node left when every start has finished.
    pub fn finish(mut self) -> GreenNode {
        debug_assert!(self.parents.is_empty() && self.children.len() == 1);
        match self.children.pop() {
            Some(GreenElement::Node(n)) => n,
            _ => GreenNode::new(NodeKind::Error, Vec::new()),
        }
    }
}

/// A node of the red tree: a green node, where it sits, and its parent.
#[derive(Clone)]
pub struct SyntaxNode(Arc<NodeData>);

struct NodeData {
    green: GreenNode,
    parent: Option<SyntaxNode>,
    offset: u32,
    /// Its place among its parent's children, tokens included.
    index: u32,
}

impl PartialEq for SyntaxNode {
    fn eq(&self, other: &SyntaxNode) -> bool {
        self.0.offset == other.0.offset && Arc::ptr_eq(&self.0.green.0, &other.0.green.0)
    }
}

impl Eq for SyntaxNode {}

impl std::hash::Hash for SyntaxNode {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.0.offset.hash(h);
        Arc::as_ptr(&self.0.green.0).hash(h);
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxToken {
    green: GreenToken,
    parent: SyntaxNode,
    offset: u32,
    index: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SyntaxElement {
    Node(SyntaxNode),
    Token(SyntaxToken),
}

impl SyntaxNode {
    pub fn new_root(green: GreenNode) -> SyntaxNode {
        SyntaxNode(Arc::new(NodeData {
            green,
            parent: None,
            offset: 0,
            index: 0,
        }))
    }

    pub fn green(&self) -> &GreenNode {
        &self.0.green
    }

    pub fn kind(&self) -> NodeKind {
        self.0.green.kind()
    }

    /// The node's bytes in the file, trivia inside it included.
    pub fn span(&self) -> Span {
        Span::new(self.0.offset, self.0.offset + self.0.green.width())
    }

    pub fn parent(&self) -> Option<SyntaxNode> {
        self.0.parent.clone()
    }

    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode> {
        std::iter::successors(Some(self.clone()), SyntaxNode::parent)
    }

    /// Its place among its parent's children, tokens included.
    pub fn index(&self) -> usize {
        self.0.index as usize
    }

    pub fn children_with_tokens(&self) -> impl Iterator<Item = SyntaxElement> + '_ {
        let mut offset = self.0.offset;
        self.0
            .green
            .children()
            .iter()
            .enumerate()
            .map(move |(index, child)| {
                let at = offset;
                offset += child.width();
                match child {
                    GreenElement::Node(green) => {
                        SyntaxElement::Node(SyntaxNode(Arc::new(NodeData {
                            green: green.clone(),
                            parent: Some(self.clone()),
                            offset: at,
                            index: index as u32,
                        })))
                    }
                    GreenElement::Token(green) => SyntaxElement::Token(SyntaxToken {
                        green: green.clone(),
                        parent: self.clone(),
                        offset: at,
                        index: index as u32,
                    }),
                }
            })
    }

    pub fn children(&self) -> impl Iterator<Item = SyntaxNode> + '_ {
        self.children_with_tokens().filter_map(|e| match e {
            SyntaxElement::Node(n) => Some(n),
            SyntaxElement::Token(_) => None,
        })
    }

    /// The node and every node under it, in preorder.
    pub fn descendants(&self) -> impl Iterator<Item = SyntaxNode> {
        let mut stack = vec![self.clone()];
        std::iter::from_fn(move || {
            let node = stack.pop()?;
            let at = stack.len();
            stack.extend(node.children());
            stack[at..].reverse();
            Some(node)
        })
    }

    /// Every token under the node, trivia included, in order.
    pub fn tokens(&self) -> impl Iterator<Item = SyntaxToken> {
        let mut stack = vec![SyntaxElement::Node(self.clone())];
        std::iter::from_fn(move || loop {
            match stack.pop()? {
                SyntaxElement::Token(t) => return Some(t),
                SyntaxElement::Node(n) => {
                    let at = stack.len();
                    stack.extend(n.children_with_tokens());
                    stack[at..].reverse();
                }
            }
        })
    }

    /// The node's text: every byte of it, trivia included.
    pub fn text(&self) -> String {
        let mut out = String::with_capacity(self.0.green.width() as usize);
        self.tokens().for_each(|t| out.push_str(t.text()));
        out
    }
}

impl fmt::Debug for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.kind(), self.span())
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tokens().try_for_each(|t| f.write_str(t.text()))
    }
}

impl SyntaxToken {
    pub fn kind(&self) -> TokenKind {
        self.green.kind()
    }

    pub fn text(&self) -> &str {
        self.green.text()
    }

    pub fn span(&self) -> Span {
        Span::new(self.offset, self.offset + self.green.width())
    }

    pub fn parent(&self) -> SyntaxNode {
        self.parent.clone()
    }

    /// Its place among its parent's children, nodes included.
    pub fn index(&self) -> usize {
        self.index as usize
    }
}

impl fmt::Debug for SyntaxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?} {:?}", self.kind(), self.span(), self.text())
    }
}
