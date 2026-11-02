use crate::{backend::node::Node, strategies::selection_strategy::BackendSelectionStrategy};

pub struct Pool {
    nodes: Vec<Node>,
    strategy: Box<dyn BackendSelectionStrategy>
}

impl Pool {
    pub fn new<T>(nodes: Vec<Node>, strategy: T) -> Result<Self, &'static str>
    where 
        T: BackendSelectionStrategy + 'static {
        if nodes.is_empty() {
            return Err("Nodes can't be empty")
        }

        Ok(Self {nodes, strategy: Box::new(strategy)})
    }

    pub fn select_backend(&self) -> &Node {
        self.strategy.select_backend(&self.nodes)
            .expect("Strategy failed to select a backend from a non-empty pool")
    }
}