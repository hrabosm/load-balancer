use crate::backend::node::Node;

pub trait BackendSelectionStrategy: Send + Sync {
    fn select_backend<'a>(&self, nodes: &'a [Node]) -> Option<&'a Node>;
}