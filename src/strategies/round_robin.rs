use std::sync::atomic::{AtomicUsize, Ordering};

use crate::strategies::selection_strategy::BackendSelectionStrategy;

pub struct RoundRobin {
    current_index: AtomicUsize,
}

impl RoundRobin {
    pub fn new() -> Self {
        RoundRobin { current_index: AtomicUsize::new(0) }
    }
}

impl BackendSelectionStrategy for RoundRobin {
    fn select_backend<'a>(&self, nodes: &'a [crate::backend::node::Node]) -> Option<&'a crate::backend::node::Node> {
        if nodes.is_empty() {
            return None
        }

        let index = self.current_index.fetch_add(1, Ordering::Relaxed);
        Some(&nodes[index % nodes.len()])
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

use crate::backend::node::Node;

    use super::*;

    #[test]
    fn fails_with_empty_nodes() {
        let nodes = Vec::<Node>::new();
        let round_robin = RoundRobin::new();

        assert!(round_robin.select_backend(&nodes).is_none());
    }

    #[test]
    fn always_selects_first_node_with_only_one_node() {
        let node = Node::from_str("http://test.com").unwrap();
        let nodes =  vec![node];
        let round_robin = RoundRobin::new();

        assert_eq!(round_robin.select_backend(&nodes).unwrap().uri(), nodes[0].uri());
        assert_eq!(round_robin.select_backend(&nodes).unwrap().uri(), nodes[0].uri());
        assert_eq!(round_robin.select_backend(&nodes).unwrap().uri(), nodes[0].uri());
    }

    #[test]
    fn cycles_through_backends_in_order() {
        let node = Node::from_str("http://test.com").unwrap();
        let node1 = Node::from_str("http://test1.com").unwrap();
        let node2 = Node::from_str("http://test2.com").unwrap();
        let node3 = Node::from_str("http://test3.com").unwrap();
        let node4 = Node::from_str("http://test4.com").unwrap();
        let nodes =  vec![node, node1, node2, node3, node4];
        let round_robin = RoundRobin::new();

        for i in 0..15 {
            assert_eq!(round_robin.select_backend(&nodes).unwrap().uri(), nodes[i%nodes.len()].uri());
        }
    }

    #[test]
    fn handles_multithreading_correctly() {
        let nodes = Arc::new(vec![
            Node::from_str("http://test.com").unwrap(),
            Node::from_str("http://test1.com").unwrap(),
            Node::from_str("http://test2.com").unwrap(),
            Node::from_str("http://test3.com").unwrap(),
            Node::from_str("http://test4.com").unwrap(),
        ]);

        let round_robin = Arc::new(RoundRobin::new());

        // One atomic counter per backend.
        let counters: Arc<Vec<AtomicUsize>> = Arc::new(
            (0..nodes.len())
                .map(|_| AtomicUsize::new(0))
                .collect(),
        );

        let mut handles = Vec::new();

        for _ in 0..10 {
            let nodes = Arc::clone(&nodes);
            let round_robin = Arc::clone(&round_robin);
            let counters = Arc::clone(&counters);

            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let selected = round_robin.select_backend(&nodes).unwrap();

                    let index = nodes
                        .iter()
                        .position(|node| node.uri() == selected.uri())
                        .unwrap();

                    counters[index].fetch_add(1, Ordering::Relaxed);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        for counter in counters.iter() {
            assert_eq!(counter.load(Ordering::Relaxed), 200);
        }
    }
}