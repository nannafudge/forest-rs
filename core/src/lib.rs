pub mod macros;
pub mod fenwick;

pub use tree::*;

/*################################
              Tree
################################*/

pub mod tree {
    pub trait Height {
        fn height(&self) -> usize;
    }

    pub trait TreeRead {
        type Node;
        type Error;
    
        fn get(&self, node: &Self::Node) -> Result<&Self::Node, Self::Error>;
        fn contains(&self, node: &Self::Node) -> Result<bool, Self::Error>;
    }

    pub trait TreeReadMut: TreeRead {
        fn get_mut(&mut self, node: &Self::Node) -> Result<&mut Self::Node, Self::Error>;
    }

    pub trait TreeWrite: TreeReadMut {
        fn insert(&mut self, node: Self::Node) -> Result<Option<Self::Node>, Self::Error>;
        fn delete(&mut self, node: &Self::Node) -> Result<Self::Node, Self::Error>;

        fn pop(&mut self) -> Result<Self::Node, Self::Error>;
    }
}

/*################################
            Tree KV
################################*/

pub mod tree_kv {
    use super::tree::{
        TreeRead, TreeReadMut,
        TreeWrite
    };

    pub use core::convert::Into;
    use core::cmp::Ordering;

    #[derive(Debug)]
    pub enum NodeKV<'a, K: PartialEq, V> {
        Occupied(K, V),
        Search(&'a K),
        None
    }

    impl<'a, K: PartialEq, V> Default for NodeKV<'a, K, V> {
        fn default() -> Self {
            NodeKV::None
        }
    }

    impl<'a, K: PartialEq, V> NodeKV<'a, K, V> {
        pub fn unwrap(self: Self) -> V {
            match self {
                NodeKV::Occupied(_, v) => v,
                _ => panic!("Invalid tree NodeKV returned from search")
            }
        }

        pub fn inner(self: &Self) -> &V {
            match self {
                NodeKV::Occupied(_, v) => v,
                _ => panic!("Invalid tree NodeKV returned from search")
            }
        }

        pub fn inner_mut(self: &mut Self) -> &mut V {
            match self {
                NodeKV::Occupied(_, v) => v,
                _ => panic!("Invalid tree NodeKV returned from search")
            }
        }
    }

    impl<'a, K, V> PartialEq<K> for NodeKV<'a, K, V> where
        K: PartialEq
    {
        fn eq(&self, other: &K) -> bool {
            match self {
                NodeKV::Occupied(k, _) => other.eq(k),
                NodeKV::Search(k) => other.eq(*k),
                NodeKV::None => false
            }
        }
    }

    impl<'a, K, V> PartialEq for NodeKV<'a, K, V> where
        K: PartialEq
    {
        fn eq(&self, other: &Self) -> bool {
            match self {
                NodeKV::Occupied(k, _) => other.eq(k),
                NodeKV::Search(k) => other.eq(*k),
                NodeKV::None => false
            }
        }
    }

    impl<'a, K, V> PartialOrd<K> for NodeKV<'a, K, V> where
        K: PartialOrd
    {
        fn partial_cmp(&self, other: &K) -> Option<Ordering> {
            match self {
                NodeKV::Occupied(k, _) => other.partial_cmp(k),
                NodeKV::Search(k) => other.partial_cmp(*k),
                NodeKV::None => None
            }
        }
    }

    impl<'a, K, V> PartialOrd for NodeKV<'a, K, V> where
        K: PartialOrd
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            match self {
                NodeKV::Occupied(k, _) => other.partial_cmp(k),
                NodeKV::Search(k) => other.partial_cmp(*k),
                NodeKV::None => None
            }
        }
    }

    pub trait TreeReadKV<'a, 'b, K, V, E>: TreeRead<Node = NodeKV<'b, K, V>, Error = E> where
        K: PartialEq + 'b,
        'b: 'a
    {
        fn get(&'a self, key: &'b K) -> Result<&'a V, E> {
            Ok(TreeRead::get(self, &NodeKV::Search(key))?.inner())
        }

        fn contains(&'a self, key: &'b K) -> Result<bool, E> {
            TreeRead::contains(self, &NodeKV::Search(key))
        }
    }

    pub trait TreeReadKVMut<'a, 'b, K, V, E>: TreeReadMut<Node = NodeKV<'b, K, V>, Error = E> where
        K: PartialEq + 'b,
        'b: 'a
    {
        fn get_mut(&'a mut self, key: &'b K) -> Result<&'a mut V, E>  {
            Ok(TreeReadMut::get_mut(self, &NodeKV::Search(key))?.inner_mut())
        }
    }

    pub trait TreeWriteKV<'a, 'b, K, V, E>: TreeWrite<Node = NodeKV<'b, K, V>, Error = E> where
        K: PartialEq + 'b,
        'b: 'a
    {
        fn insert(&'a mut self, key: K, value: V) -> Result<Option<V>, E> {
            Ok(
                TreeWrite::insert(self, NodeKV::Occupied(key, value))?
                    .map(| kv  | kv.unwrap())
            )
        }

        fn delete(&'a mut self, key: &'b K) -> Result<V, E> {
            Ok(TreeWrite::delete(self, &NodeKV::Search(key))?.unwrap())
        }

        fn pop(&'a mut self) -> Result<NodeKV<K, V>, E> {
            TreeWrite::pop(self)
        }
    }

    impl<'a, 'b: 'a, K: PartialEq + 'b, V, E, T> TreeReadKV<'a, 'b, K, V, E> for T where
        T: TreeRead<Node = NodeKV<'b, K, V>, Error = E> {}
    impl<'a, 'b: 'a, K: PartialEq + 'b, V, E, T> TreeReadKVMut<'a, 'b, K, V, E> for T where
        T: TreeReadMut<Node = NodeKV<'b, K, V>, Error = E> {}
    impl<'a, 'b: 'a, K: PartialEq + 'b, V, E, T> TreeWriteKV<'a, 'b, K, V, E> for T where
        T: TreeWrite<Node = NodeKV<'b, K, V>, Error = E> + 'a {}
}

/*################################
           Tree Walker 
################################*/

pub trait TreeWalker<'w> {
    type Path;
    type Output;
    type Error;

    fn peek(&'w self, direction: Direction) -> Result<Self::Output, Self::Error>;
    fn probe(&'w self, path: Self::Path) -> Result<Self::Output, Self::Error>;
    fn current(&'w self) -> Result<Self::Output, Self::Error>;
    fn sibling(&'w self) -> Result<Self::Output, Self::Error>;

    fn traverse(&'w mut self, direction: Direction);
    fn seek(&'w mut self, path: Self::Path);
    fn reset(&'w mut self);
    
    fn type_(&'w self) -> NodeType;
    fn side(&'w self) -> NodeSide;
}

pub trait TreeWalkerMut<'w>: TreeWalker<'w> {
    type OutputMut;

    fn peek_mut(&'w mut self, direction: Direction) -> Result<Self::OutputMut, Self::Error>;
    fn probe_mut(&'w mut self, path: Self::Path) -> Result<Self::OutputMut, Self::Error>;

    fn current_mut(&'w mut self) -> Result<Self::OutputMut, Self::Error>;
    fn sibling_mut(&'w mut self) -> Result<Self::OutputMut, Self::Error>;
}

pub enum Direction {
    Up,
    Down(NodeSide),
    Left,
    Right
}

/*################################
            Tree Node
################################*/

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeSide {
    Left,
    Right
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeType {
    Node,
    Leaf
}