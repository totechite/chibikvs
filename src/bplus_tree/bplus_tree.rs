use std::{convert::TryFrom, fmt::Debug, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

pub(crate) const B: usize = 3;
pub(crate) const MIN_LEN: usize = B - 1;
pub(crate) const CAPACITY: usize = 2 * B - 1;
pub(crate) const INTERNAL_CHILDREN_CAPACITY: usize = CAPACITY+1;

pub mod marker {
    #[derive(Debug)]
    pub enum Leaf {}
    #[derive(Debug)]
    pub enum Internal {}
    #[derive(Debug)]
    pub enum LeafOrInternal {}
}

#[derive(Debug)]
pub enum ForceResult<Leaf, Internal> {
    Leaf(Leaf),
    Internal(Internal),
}

#[derive(Debug)]
pub(crate) enum InsertBehavior<K: Debug, V: Debug> {
    Split(K, NodeRef<K, V, marker::LeafOrInternal>),
    Fit,
}

#[derive(Debug)]
pub struct BPlusTree<K: Debug, V: Debug> {
    pub(crate) root: Root<K, V>,
}

unsafe impl<K: Ord + Debug, V: Debug> Sync for BPlusTree<K, V> {}
unsafe impl<K: Ord + Debug, V: Debug> Send for BPlusTree<K, V> {}

impl<K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn new() -> Self {
        BPlusTree { root: Root::new() }
    }
}

#[derive(Debug)]
pub(crate) struct Root<K: Debug, V: Debug> {
    pub(crate) root: NodeRef<K, V, marker::LeafOrInternal>,
}

#[derive(Debug, Clone)]
pub(crate) struct BoxedNode<K: Debug, V: Debug> {
    pub(crate) ptr: NonNull<LeafNode<K, V>>,
}

#[derive(Debug)]
pub(crate) struct BoxedKey<'a, K: Debug> {
    content: &'a K,
}

#[derive(Debug)]
pub(crate) struct NodeRef<K: Debug, V: Debug, NodeType> {
    pub(crate) height: u16,
    pub(crate) node: BoxedNode<K, V>,
    pub(crate) _metatype: PhantomData<NodeType>,
}

unsafe impl<K: Ord + Debug, V: Debug, Type> Sync for NodeRef<K, V, Type> {}
unsafe impl<K: Ord + Debug, V: Debug, Type> Send for NodeRef<K, V, Type> {}

#[derive(Debug)]
pub(crate) struct InternalNode<K: Debug, V: Debug> {
    pub(crate) keys: [MaybeUninit<K>; CAPACITY],
    pub(crate) length: u16,
    pub(crate) children: [MaybeUninit<NodeRef<K, V, marker::LeafOrInternal>>; INTERNAL_CHILDREN_CAPACITY],
}
unsafe impl<'a, K: Ord + Debug, V: Debug> Sync for InternalNode<K, V> {}
unsafe impl<'a, K: Ord + Debug, V: Debug> Send for InternalNode<K, V> {}

#[derive(Debug)]
pub(crate) struct LeafNode<K: Debug, V: Debug> {
    pub(crate) keys: [MaybeUninit<K>; CAPACITY],
    pub(crate) vals: [MaybeUninit<V>; CAPACITY],
    pub(crate) length: u16,
    pub(crate) prev_leaf: Option<*const Self>,
    pub(crate) next_leaf: Option<*const Self>,
}
unsafe impl<K: Ord + Debug, V: Debug> Sync for LeafNode<K, V> {}
unsafe impl<K: Ord + Debug, V: Debug> Send for LeafNode<K, V> {}

impl<K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn values(&self) -> Vec<V> {
        self.root.root.traverse_values()
    }

    pub fn keys(&self) -> Vec<K> {
        self.root.root.traverse_keys()
    }
}

impl<'a, K: Debug, V: Debug> NodeRef<K, V, marker::LeafOrInternal> {
    pub(crate) fn force(
        &'a self,
    ) -> ForceResult<NodeRef<K, V, marker::Leaf>, NodeRef<K, V, marker::Internal>> {
        let boxed_node = BoxedNode::<K, V> {
            ptr: self.node.as_ptr(),
        };
        if self.height == 0 {
            ForceResult::Leaf(NodeRef {
                height: self.height,
                node: boxed_node,
                _metatype: PhantomData,
            })
        } else {
            ForceResult::Internal(NodeRef {
                height: self.height,
                node: boxed_node,
                _metatype: PhantomData,
            })
        }
    }
}

impl<K: Debug, V: Debug> NodeRef<K, V, marker::Leaf> {
    pub(crate) fn from_boxed_node(boxednode: BoxedNode<K, V>) -> Self {
        Self {
            node: boxednode,
            height: 0,
            _metatype: PhantomData,
        }
    }
}

impl<'a, K: Debug, V: Debug> NodeRef<K, V, marker::Internal> {
    pub(crate) fn from_boxed_node(boxednode: BoxedNode<K, V>) -> Self {
        Self {
            node: boxednode,
            height: 0,
            _metatype: PhantomData,
        }
    }

    pub(crate) fn as_internal(&self) -> &'a InternalNode<K, V> {
        unsafe {
            &std::mem::transmute::<&LeafNode<K, V>, &InternalNode<K, V>>(&self.node.ptr.as_ref())
        }
    }
    pub(crate) fn as_internal_mut(&mut self) -> &'a mut InternalNode<K, V> {
        unsafe {
            std::mem::transmute::<&mut LeafNode<K, V>, &mut InternalNode<K, V>>(
                &mut self.node.ptr.as_mut(),
            )
        }
    }
    pub(crate) fn up_cast(self) -> NodeRef<K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _metatype: PhantomData,
        }
    }
}

impl<K: Debug, V: Debug> BoxedNode<K, V> {
    pub(crate) fn from_leaf(node: Box<LeafNode<K, V>>) -> Self {
        BoxedNode {
            ptr: NonNull::from(Box::leak(node)),
        }
    }

    pub(crate) fn from_internal(node: Box<InternalNode<K, V>>) -> Self {
        BoxedNode {
            ptr: NonNull::from(Box::leak(node)).cast(),
        }
    }

    pub(crate) fn as_ptr(&self) -> NonNull<LeafNode<K, V>> {
        NonNull::from(self.ptr)
    }
}

impl<'a, K: Debug, V: Debug> InternalNode<K, V> {
    pub(crate) fn new() -> Self {
        InternalNode {
            keys: MaybeUninit::uninit_array(),
            length: 0,
            children: MaybeUninit::uninit_array(),
        }
    }
}

impl<K: Debug, V: Debug> LeafNode<K, V> {
    pub(crate) fn new() -> Self {
        LeafNode {
            keys: MaybeUninit::uninit_array(),
            vals: MaybeUninit::uninit_array(),
            length: 0,
            prev_leaf: None,
            next_leaf: None,
        }
    }
}

impl<K: Ord + Debug, V: Debug> Root<K, V> {
    pub(crate) fn new() -> Self {
        let leaf = Box::new(LeafNode::new());
        let root = NodeRef::<K, V, marker::LeafOrInternal> {
            height: 0,
            node: BoxedNode::from_leaf(leaf),
            _metatype: PhantomData,
        };

        Root { root }
    }
}

impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::LeafOrInternal> {
    fn traverse_values(&self) -> Vec<V> {
        match self.force() {
            ForceResult::Leaf(node) => node.traverse_values(),
            ForceResult::Internal(node) => node.traverse_values(),
        }
    }

    fn traverse_keys(&self) -> Vec<K> {
        match self.force() {
            ForceResult::Leaf(node) => node.traverse_keys(),
            ForceResult::Internal(node) => node.traverse_keys(),
        }
    }
}

impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::Leaf> {
    fn traverse_values(&self) -> Vec<V> {
        let leaf = unsafe { self.node.ptr.as_ref() };
        return unsafe { leaf.traverse_values() };
    }

    fn traverse_keys(&self) -> Vec<K> {
        let leaf = unsafe { self.node.ptr.as_ref() };
        return unsafe { leaf.traverse_keys() };
    }
}

impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::Internal> {
    pub(crate) fn cut_right(&mut self) -> (K, Box<InternalNode<K, V>>) {
        self.as_internal_mut().cut_right()
    }

    pub(crate) fn split(&mut self) -> (Box<InternalNode<K, V>>, K, Box<InternalNode<K, V>>) {
        self.as_internal_mut().split()
    }

    fn traverse_values(&self) -> Vec<V> {
        self.as_internal().traverse_values()
    }
    fn traverse_keys(&self) -> Vec<K> {
        self.as_internal().traverse_keys()
    }

    pub(crate) unsafe fn join_node(
        &mut self,
        index: usize,
        key: K,
        node: NodeRef<K, V, marker::LeafOrInternal>,
    ) {
        let mut self_as_internal = self.as_internal_mut();
        let mut key = MaybeUninit::new(key);
        let mut node = MaybeUninit::new(node);

        for idx in index..self_as_internal.length() {
            std::mem::swap(&mut self_as_internal.keys[idx], &mut key);
        }
        for idx in (index + 1)..self_as_internal.length() + 1 {
            std::mem::swap(&mut self_as_internal.children[idx], &mut node);
        }

        self_as_internal.length += 1;
    }
}

impl<K: Ord + Debug, V: Debug> LeafNode<K, V> {
    pub(crate) fn length(&self) -> usize {
        self.length as usize
    }

    unsafe fn traverse_values(&self) -> Vec<V> {
        let mut current_leaf_vals = self.vals[0..self.length()]
            .iter()
            .map(|x| x.assume_init_read())
            .collect::<Vec<V>>();
        if let Some(next) = self.next_leaf {
            let mut ret = std::ptr::read(next).traverse_values();
            current_leaf_vals.append(&mut ret);
        }
        current_leaf_vals
    }

    unsafe fn traverse_keys(&self) -> Vec<K> {
        let mut current_leaf_keys = self.keys[0..self.length()]
            .iter()
            .map(|x| x.assume_init_read())
            .collect::<Vec<K>>();
        if let Some(next) = self.next_leaf {
            let mut ret = std::ptr::read(next).traverse_keys();
            current_leaf_keys.append(&mut ret);
        }
        current_leaf_keys
    }

    pub(crate) fn split(&mut self) -> (Box<LeafNode<K, V>>, Box<LeafNode<K, V>>) {
        let mut left_leafnode = LeafNode::new();
        let mut right_leafnode = LeafNode::new();

        for idx in 0..B {
            std::mem::swap(&mut right_leafnode.keys[idx], &mut self.keys[B - 1 + idx]);
            std::mem::swap(&mut right_leafnode.vals[idx], &mut self.vals[B - 1 + idx]);
        }
        right_leafnode.length = TryFrom::try_from(B).unwrap();

        for idx in 0..B - 1 {
            std::mem::swap(&mut left_leafnode.keys[idx], &mut self.keys[idx]);
            std::mem::swap(&mut left_leafnode.vals[idx], &mut self.vals[idx]);
        }
        left_leafnode.length = TryFrom::try_from(CAPACITY - B).unwrap();
        right_leafnode.prev_leaf = Some(&mut left_leafnode as *mut LeafNode<K, V>);
        right_leafnode.next_leaf = self.next_leaf.take();

        left_leafnode.prev_leaf = self.prev_leaf.take();
        left_leafnode.next_leaf = Some(&mut right_leafnode as *mut LeafNode<K, V>);
        (Box::new(left_leafnode), Box::new(right_leafnode))
    }
}

impl<'a, K: Ord + Debug, V: Debug> InternalNode<K, V> {
    pub(crate) fn length(&'a self) -> usize {
        self.length as usize
    }

    fn set_length(&'a mut self, len: u16) {
        self.length = len;
    }

    pub(crate) fn cut_right(&'a mut self) -> (K, Box<InternalNode<K, V>>) {
        let mut right_internal_node: InternalNode<K, V> = InternalNode::new();

        let raised_key = unsafe { self.keys[B - 1].assume_init_read() };

        for idx in 0..B - 1 {
            std::mem::swap(&mut right_internal_node.keys[idx], &mut self.keys[B + idx]);
        }
        for idx in 0..B {
            std::mem::swap(
                &mut right_internal_node.children[idx],
                &mut self.children[B + idx],
            );
        }
        self.length = B as u16;
        right_internal_node.length = B as u16;

        (raised_key, Box::new(right_internal_node))
    }

    pub(crate) fn split(&'a mut self) -> (Box<InternalNode<K, V>>, K, Box<InternalNode<K, V>>) {
        let mut left_internal_node: InternalNode<K, V> = InternalNode::new();
        let mut right_internal_node: InternalNode<K, V> = InternalNode::new();

        let raised_key = unsafe { self.keys[B - 1].assume_init_read() };

        for idx in 0..B - 1 {
            std::mem::swap(&mut left_internal_node.keys[idx], &mut self.keys[idx]);
        }
        for idx in 0..B - 1 {
            std::mem::swap(&mut right_internal_node.keys[idx], &mut self.keys[B + idx]);
        }
        for idx in 0..B {
            std::mem::swap(
                &mut left_internal_node.children[idx],
                &mut self.children[idx],
            );
        }
        for idx in 0..B {
            std::mem::swap(
                &mut right_internal_node.children[idx],
                &mut self.children[B + idx],
            );
        }

        left_internal_node.length = B as u16;
        right_internal_node.length = B as u16;

        (
            Box::new(left_internal_node),
            raised_key,
            Box::new(right_internal_node),
        )
    }

    fn traverse_values(&self) -> Vec<V> {
        unsafe { self.children[0].assume_init_ref().traverse_values() }
    }

    fn traverse_keys(&self) -> Vec<K> {
        unsafe { self.children[0].assume_init_ref().traverse_keys() }
    }
}
