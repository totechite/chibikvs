use std::{convert::TryFrom, fmt::Debug, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

const B: usize = 3;
pub const MIN_LEN: usize = B - 1;
pub const CAPACITY: usize = 2 * B - 1;

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
enum InsertBehavior<K: Debug, V: Debug> {
    Split(K, NodeRef<K, V, marker::LeafOrInternal>),
    Fit,
}

#[derive(Debug)]
pub struct BPlusTree<K: Debug, V: Debug> {
    root: Root<K, V>,
}

impl<K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn new() -> Self {
        BPlusTree { root: Root::new() }
    }
}

#[derive(Debug)]
struct Root<K: Debug, V: Debug> {
    pub root: NodeRef<K, V, marker::LeafOrInternal>,
}

#[derive(Debug)]
struct BoxedNode<K: Debug, V: Debug> {
    ptr: NonNull<LeafNode<K, V>>,
}

#[derive(Debug)]
pub struct NodeRef<K: Debug, V: Debug, NodeType> {
    height: u16,
    node: BoxedNode<K, V>,
    _metatype: PhantomData<NodeType>,
}

unsafe impl<K: Ord + Debug, V: Debug, Type> Sync for NodeRef<K, V, Type> {}
unsafe impl<K: Ord + Debug, V: Debug, Type> Send for NodeRef<K, V, Type> {}

// impl<K: Ord + Debug, V: Debug, Type> Clone for NodeRef<K, V, Type> {
//     fn clone(&self) -> Self {
//         NodeRef {
//             node: self.node,
//             height: self.height,
//             _metatype: self._metatype,
//         }
//     }
// }

impl<K: Debug, V: Debug> NodeRef<K, V, marker::LeafOrInternal> {
    fn force(&self) -> ForceResult<NodeRef<K, V, marker::Leaf>, NodeRef<K, V, marker::Internal>> {
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

impl<'a, K: Debug, V: Debug> NodeRef<K, V, marker::Internal> {
    fn as_internal(&self) -> &InternalNode<'a, K, V> {
        unsafe {
            &std::mem::transmute::<&LeafNode<K, V>, &InternalNode<K, V>>(&self.node.ptr.as_ref())
        }
    }
    fn as_internal_mut(&mut self) -> &mut InternalNode<'a, K, V> {
        unsafe {
            std::mem::transmute::<&mut LeafNode<K, V>, &mut InternalNode<K, V>>(
                &mut self.node.ptr.as_mut(),
            )
        }
    }
}

impl<K: Debug, V: Debug> BoxedNode<K, V> {
    fn from_leaf(node: Box<LeafNode<K, V>>) -> Self {
        BoxedNode {
            ptr: NonNull::from(Box::leak(node)),
        }
    }

    fn from_internal(node: Box<InternalNode<K, V>>) -> Self {
        BoxedNode {
            ptr: NonNull::from(Box::leak(node)).cast(),
        }
    }

    fn as_ptr(&self) -> NonNull<LeafNode<K, V>> {
        NonNull::from(self.ptr)
    }
}

#[derive(Debug)]
pub struct InternalNode<'a, K: Debug, V: Debug> {
    keys: [MaybeUninit<&'a K>; CAPACITY],
    length: u16,
    children: [MaybeUninit<NodeRef<K, V, marker::LeafOrInternal>>; CAPACITY + 1],
}

unsafe impl<'a, K: Ord + Debug, V: Debug> Sync for InternalNode<'a, K, V> {}
unsafe impl<'a, K: Ord + Debug, V: Debug> Send for InternalNode<'a, K, V> {}

impl<'a, K: Debug, V: Debug> InternalNode<'a, K, V> {
    fn new() -> Self {
        InternalNode {
            keys: MaybeUninit::uninit_array(),
            length: 0,
            children: MaybeUninit::uninit_array(),
        }
    }
}

#[derive(Debug)]
pub struct LeafNode<K: Debug, V: Debug> {
    pub keys: [MaybeUninit<K>; CAPACITY],
    pub vals: [MaybeUninit<V>; CAPACITY],
    pub length: u16,
    pub prev_leaf: Option<*const LeafNode<K, V>>,
    pub next_leaf: Option<*const LeafNode<K, V>>,
}

impl<K: Debug, V: Debug> LeafNode<K, V> {
    fn new() -> Self {
        LeafNode {
            keys: MaybeUninit::uninit_array(),
            vals: MaybeUninit::uninit_array(),
            length: 0,
            prev_leaf: None,
            next_leaf: None,
        }
    }
}

unsafe impl<K: Ord + Debug, V: Debug> Sync for LeafNode<K, V> {}
unsafe impl<K: Ord + Debug, V: Debug> Send for LeafNode<K, V> {}

impl<K: Ord + Debug, V: Debug> Root<K, V> {
    pub fn new() -> Self {
        let mut leaf = Box::new(LeafNode::new());
        let root = NodeRef::<K, V, marker::LeafOrInternal> {
            height: 0,
            node: unsafe { BoxedNode::from_leaf(leaf) },
            _metatype: PhantomData,
        };

        Root { root }
    }
}

impl<K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.root.insert(key, value)
    }
}

impl<'a, K: Ord + Sized + Debug + 'a, V: Debug> Root<K, V> {
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let (behavior, ret) = self.root.insert(key, value);
        match behavior {
            InsertBehavior::Fit => {}
            InsertBehavior::Split(key, left_child) => {
                let mut new_root = Box::new(InternalNode::<K, V>::new());

                let right_child = {
                    let node_ptr = self.root.node.as_ptr().as_ptr();
                    let boxed_node = BoxedNode::from_leaf(unsafe { Box::from_raw(node_ptr) });
                    NodeRef::<K, V, marker::LeafOrInternal> {
                        node: boxed_node,
                        height: self.root.height,
                        _metatype: PhantomData,
                    }
                };
                unsafe {
                    println!("{:?}", key);
                    new_root.keys[0] = MaybeUninit::new(&key);
                }

                unsafe {
                    println!(
                        "{:?}",
                        MaybeUninit::slice_assume_init_ref(&self.root.node.as_ptr().as_ref().keys)
                    );
                    println!("{:?}", &self.root.node.as_ptr().as_ref().length());
                    println!(
                        "{:?}",
                        MaybeUninit::slice_assume_init_ref(&left_child.node.as_ptr().as_ref().keys)
                    );
                    println!("{:?}", &left_child.node.as_ptr().as_ref().length());
                }

                new_root.children[0] = MaybeUninit::new(right_child);
                new_root.children[1] = MaybeUninit::new(left_child);
                new_root.length = 2;
                println!("{:?}", new_root);

                self.root.node = BoxedNode::from_internal(new_root);
                // let new_root_noderef = NodeRef::<K, V, marker::LeafOrInternal> {
                //     node: NonNull::new(
                //         (&mut new_root as *mut InternalNode<K, V>) as *mut LeafNode<K, V>,
                //     )
                //     .unwrap(),
                //     height: self.root.height + 1,
                //     root: None,
                //     _metatype: PhantomData,
                // };

                self.root.height += 1;
            }
        }
        return ret;
    }
}

impl<K: Ord + Debug, V: Debug> NodeRef<K, V, marker::LeafOrInternal> {
    fn insert(&mut self, key: K, value: V) -> (InsertBehavior<K, V>, Option<V>) {
        unsafe {
            match self.force() {
                ForceResult::Leaf(mut node) => {
                    match node.insert(key, value) {
                        (InsertBehavior::Fit, option) => {
                            return (InsertBehavior::Fit, option);
                        }
                        (InsertBehavior::Split(key, left_child), option) => {
                            return (InsertBehavior::Split(key, left_child), option);
                        }
                    };
                }
                ForceResult::Internal(mut node) => {
                    match node.insert(key, value) {
                        (InsertBehavior::Fit, option, idx) => {
                            return (InsertBehavior::Fit, option);
                        }
                        (InsertBehavior::Split(key, left_child), option, idx) => {
                            node.join_node(idx, &key, left_child);
                            return (InsertBehavior::Fit, option);
                        }
                    };
                }
            }

            // return if self.height == 0 {
            //     let mut leaf = self.node.ptr.as_mut();
            //     leaf.insert(key, value)
            // } else {
            //     let mut internal = unsafe {
            //         std::mem::transmute::<&mut LeafNode<K, V>, &mut InternalNode<K, V>>(
            //             &mut self.node.as_ptr().as_mut(),
            //         )
            //     };
            //     internal.insert(key, value)
            // };
        }
    }
}

impl<K: Ord + Debug, V: Debug> NodeRef<K, V, marker::Leaf> {
    unsafe fn insert(&mut self, key: K, value: V) -> (InsertBehavior<K, V>, Option<V>) {
        let leaf = self.node.ptr.as_mut();
        return leaf.insert(key, value);
    }
}

impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::Internal> {
    fn split(&mut self) -> (Box<InternalNode<'a, K, V>>, Box<InternalNode<'a, K, V>>) {
        self.as_internal_mut().split()
    }

    unsafe fn insert(&mut self, key: K, value: V) -> (InsertBehavior<K, V>, Option<V>, usize) {
        let internal = self.as_internal_mut();
        return internal.insert(key, value);
    }

    unsafe fn join_node(
        &mut self,
        index: usize,
        key: &K,
        node: NodeRef<K, V, marker::LeafOrInternal>,
    ) {
        let mut internal = self.as_internal_mut();
        let mut key = MaybeUninit::new(key);
        let mut node = MaybeUninit::new(node);
        println!("{:?}", internal.length());
        println!("index: {:?}", index);
        println!("raised key: {:?}", key.assume_init_ref());

        for idx in index..internal.length() {
            std::mem::swap(&mut internal.keys[idx], &mut key);
        }
        for idx in index..internal.length() + 1 {
            std::mem::swap(&mut internal.children[idx], &mut node);
        }

        internal.length += 1;
    }
}

impl<K: Ord + Debug, V: Debug> LeafNode<K, V> {
    fn length(&self) -> usize {
        self.length as usize
    }

    fn split(&mut self) -> (Box<LeafNode<K, V>>, Box<LeafNode<K, V>>) {
        let mut left_leafnode = LeafNode::new();
        let mut right_leafnode =LeafNode::new();

        for idx in 0..B {
            std::mem::swap(&mut right_leafnode.keys[idx], &mut self.keys[B - 1 + idx]);
            std::mem::swap(&mut right_leafnode.vals[idx], &mut self.vals[B - 1 + idx]);
        }
        for idx in 0..B - 1 {
            std::mem::swap(&mut left_leafnode.keys[idx], &mut self.keys[idx]);
            std::mem::swap(&mut left_leafnode.vals[idx], &mut self.vals[idx]);
        }

        right_leafnode.length = TryFrom::try_from(B).unwrap();
        right_leafnode.prev_leaf = Some(&mut left_leafnode as *const Self);
        right_leafnode.next_leaf = self.next_leaf;

        left_leafnode.length = TryFrom::try_from(CAPACITY - B).unwrap();
        left_leafnode.prev_leaf = self.prev_leaf;
        left_leafnode.next_leaf = Some(&mut right_leafnode as  *mut Self);

      
        (Box::new(left_leafnode), Box::new(right_leafnode))
    }

    fn insert(&mut self, key: K, value: V) -> (InsertBehavior<K, V>, Option<V>) {
        // println!("Leaf: self.length == {:?}, Key: {:?}", self.length, &key);
        // unsafe {
        //     println!("LeafNode.insert(): self.length {:?}", &self.length());
        // }

        // println!("insert key: {:?}", key);

        if self.length() < CAPACITY {
            // 空きがある場合

            if let Some(idx) = self
                .keys
                .iter()
                .position(|x| unsafe { x.assume_init_ref() == &key })
            {
                // 既存のkeyで挿入される場合、新しいvalueと古いvalueが交換され、古いvalueが戻り値となる。

                let mut swaped_val: MaybeUninit<V> = MaybeUninit::new(value);
                std::mem::swap(&mut self.vals[idx], &mut swaped_val);
                let ret: V = unsafe { swaped_val.assume_init() };
                return (InsertBehavior::Fit, Some(ret));
            } else {
                // 新規のkeyの場合、挿入位置を決定する。戻り値はNone。
                for idx in 0..self.length() {
                    let next = unsafe { self.keys[idx].assume_init_ref() };
                    println!("compared key: {:?}", next);
                    if &key < next {
                        // idx番目から要素を詰める
                        let mut inserted_key = MaybeUninit::new(key);
                        let mut inserted_val = MaybeUninit::new(value);
                        for idx in idx..self.length() + 1 {
                            std::mem::swap(&mut self.keys[idx], &mut inserted_key);
                            std::mem::swap(&mut self.vals[idx], &mut inserted_val);
                        }
                        self.length += 1;
                        return (InsertBehavior::Fit, None);
                    }
                }
                // ノードが保持するどのkeyよりも大きいkeyとして取り扱う。
                let inserted_key = MaybeUninit::new(key);
                let inserted_val = MaybeUninit::new(value);
                self.keys[self.length()] = inserted_key;
                self.vals[self.length()] = inserted_val;
                self.length += 1;
            }
            return (InsertBehavior::Fit, None);
        } else {
            //　空きがない場合
            println!("{:?}", "Leaf is splited");

            let mut new_leafnode = LeafNode {
                keys: MaybeUninit::uninit_array(),
                vals: MaybeUninit::uninit_array(),
                length: TryFrom::try_from(B).unwrap(),
                prev_leaf:  Some(self as *mut LeafNode<K,V>),
                next_leaf: self.next_leaf,
            };


            for idx in 0..B {
                std::mem::swap(&mut new_leafnode.keys[idx], &mut self.keys[B - 1 + idx]);
                std::mem::swap(&mut new_leafnode.vals[idx], &mut self.vals[B - 1 + idx]);
            }

            self.length = TryFrom::try_from(CAPACITY.wrapping_sub(B)).unwrap();

            unsafe {
                if &key <= self.keys[self.length() - 1].assume_init_ref() {
                    self.insert(key, value);
                } else {
                    new_leafnode.insert(key, value);
                };
            }

            // unsafe {
            //     println!("{:?}", MaybeUninit::slice_assume_init_ref(&self.keys));
            //     println!("{:?}", self.length());
            //     println!(
            //         "{:?}",
            //         MaybeUninit::slice_assume_init_ref(&new_leafnode.keys)
            //     );
            //     println!("{:?}", new_leafnode.length());
            // }

            self.next_leaf = Some(&mut new_leafnode as *mut LeafNode<K,V>);


            let new_noderef = NodeRef {
                node: BoxedNode::from_leaf(Box::new(new_leafnode)),
                height: 0,
                _metatype: PhantomData,
            };


            unsafe {
                // println!("{:?}", self.keys[self.length() - 1].assume_init_read());
                let shaft_key = self.keys[self.length() - 1].assume_init_read();
                return (InsertBehavior::Split(shaft_key, new_noderef), None);
            }
        }
    }
}

impl<'a, K: Ord + Debug, V: Debug> InternalNode<'a, K, V> {
    fn length(&self) -> usize {
        self.length as usize
    }

    fn split(&mut self) -> (Box<InternalNode<'a, K, V>>, Box<InternalNode<'a, K, V>>) {
        let mut left_internal_node: InternalNode<K, V> = InternalNode::new();
        let mut right_internal_node: InternalNode<K, V> = InternalNode::new();

        let raised_key = unsafe {self.keys[B - 1].assume_init_read()};

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
        left_internal_node.length = 3;
        right_internal_node.length = 3;

        (Box::new(left_internal_node), Box::new(right_internal_node))
    }

    fn insert(&mut self, key: K, value: V) -> (InsertBehavior<K, V>, Option<V>, usize) {
        unsafe {
            println!(
                "Internal: self.length == {:?}, Key: {:?}",
                self.length(),
                &key
            );
            println!("Internal insert key: {:?}", key);

            if self.length() < CAPACITY + 1 {
                // 空きがある場合

                // unsafe {
                //     println!("{:?}", MaybeUninit::slice_assume_init_ref(&self.keys));
                //     println!("{:?}", self.length());
                // }

                for idx in 0..self.length() - 1 {
                    // 挿入位置を決定する。

                    let next = self.keys[idx].assume_init_ref();
                    println!("Internal compared key: {:?}", next);
                    if &key <= next {
                        return {
                            let (insert_behavior, option) =
                                self.children[idx].assume_init_mut().insert(key, value);
                            (insert_behavior, option, idx)
                        };
                    }
                }

                // ノードが保持するどのkeyよりも大きいkeyとして取り扱う。

                let idx = self.length() - 1;
                let (insert_behavior, option) =
                    self.children[idx].assume_init_mut().insert(key, value);
                // println!("Internal: self.length == {:?}", self.length());
                return (insert_behavior, option, idx);
            } else {
                //　空きがない場合

                let mut new_internal_node: InternalNode<K, V> = InternalNode::new();

                let raised_key = self.keys[B].assume_init_read();

                for idx in 0..B - 1 {
                    std::mem::swap(&mut new_internal_node.keys[idx], &mut self.keys[B + idx]);
                }
                for idx in 0..B {
                    std::mem::swap(
                        &mut new_internal_node.children[idx],
                        &mut self.children[B + idx],
                    );
                }
                new_internal_node.length = 3;

                unsafe {
                    if &key <= self.keys[self.length() - 1].assume_init_ref() {
                        self.insert(key, value);
                    } else {
                        new_internal_node.insert(key, value);
                    };
                }

                let new_noderef = NodeRef::<K, V, marker::LeafOrInternal> {
                    node: BoxedNode::from_internal(Box::new(new_internal_node)),
                    height: 0,
                    _metatype: PhantomData,
                };

                // return (InsertBehavior::Split(raised_key, new_noderef), None, 0);

                // Todo
                unimplemented!("internalnodeに空きがない場合")
            }
        }
    }
}
