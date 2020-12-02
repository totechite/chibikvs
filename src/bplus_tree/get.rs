use crate::bplus_tree::bplus_tree::*;
use std::{convert::TryFrom, fmt::Debug, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

impl<'a, K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn get(&self, key: &'a K) -> Option<&V> {
        let v = self.root.get(key)?;
        Some(unsafe { &*v })
    }
}

impl<'a, K: Ord + Sized + Debug, V: Debug> Root<K, V> {
    pub(crate) fn get(&'a self, key: &K) -> Option<*const V> {
        self.root.get(key)
    }
}

impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::LeafOrInternal> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        match self.force() {
            ForceResult::Leaf(node) => node.get(key),
            ForceResult::Internal(node) => node.as_internal().get(key),
        }
    }
}

impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::Internal> {
    pub(crate) fn get(&mut self, key: &K) -> Option<*const V> {
        let internal = self.as_internal();
        return internal.get(key);
    }
}
impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::Leaf> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        let leaf = unsafe { self.node.ptr.as_ref() };
        return leaf.get(key);
    }
}

impl<'a, K: Ord + Debug, V: Debug> InternalNode<K, V> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        for idx in 0..self.length() - 1 {
            // 挿入位置を決定する。
            let next = unsafe { self.keys[idx].assume_init_read() };
            if key <= &next {
                return unsafe { self.children[idx].assume_init_ref().get(key) };
            }
        }

        // ノードが保持するどのkeyよりも大きいkeyとして取り扱う。
        let idx = self.length() - 1;
        return unsafe { self.children[idx].assume_init_ref().get(key) };
    }
}

impl<'a, K: Ord + Debug, V: Debug> LeafNode<K, V> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        let matching_key = |x: &MaybeUninit<K>| unsafe { x.assume_init_ref() == key };
        let idx = self.keys.iter().position(matching_key)?;
        return Some(unsafe { (self.vals[idx].as_ptr()) });
    }
}
