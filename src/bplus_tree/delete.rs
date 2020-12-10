use crate::bplus_tree::bplus_tree::*;
use std::{fmt::Debug, mem::MaybeUninit};

impl<'a, K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub(crate) fn delete(&mut self, key: &K) -> Option<V> {
        let ret = self.root.delete(key);
        self.length -= 1;
        return ret;
    }
}

impl<'a, K: Ord + Sized + Debug, V: Debug> Root<K, V> {
    pub(crate) fn delete(&mut self, key: &K) -> Option<V> {
        let (_, value) = self.root.delete(key)?;
        return Some(value);
    }
}

impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::LeafOrInternal> {
    pub(crate) fn delete(&mut self, key: &K) -> Option<(usize, V)> {
        let delete_behavier = match self.force() {
            ForceResult::Leaf(mut node) => node.delete(key),
            ForceResult::Internal(mut node) => node.delete(key),
        };
        match delete_behavier {
            DeleteBehavior::Success(ret) => return ret,
            DeleteBehavior::RaiseChild(node, ret) => {
                self.node = node.node;
                self.height = node.height;
                return ret;
            }
        }
    }

    pub(crate) fn get_largest_key(&self) -> K {
        match self.force() {
            ForceResult::Leaf(node) => node.get_largest_key(),
            ForceResult::Internal(node) => node.get_largest_key(),
        }
    }

    pub(crate) fn devide(&mut self, node: &mut Self) -> bool {
        match (self.force(), node.force()) {
            (ForceResult::Leaf(mut devided), ForceResult::Leaf(mut supplied)) => {
                devided.devide(&mut supplied)
            }
            (ForceResult::Internal(mut devided), ForceResult::Internal(mut supplied)) => {
                devided.devide(&mut supplied)
            }
            _ => panic!(),
        }
    }

    pub(crate) fn marge(&mut self, node: &mut Self) {
        match (self.force(), node.force()) {
            (ForceResult::Leaf(mut margeed), ForceResult::Leaf(mut marge_node)) => {
                margeed.marge(&mut marge_node)
            }
            (ForceResult::Internal(mut margeed), ForceResult::Internal(mut marge_node)) => {
                margeed.marge(&mut marge_node)
            }
            _ => panic!(),
        }
    }
}

impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::Internal> {
    pub(crate) fn delete(&mut self, key: &K) -> DeleteBehavior<K, V> {
        let internal = self.as_internal_mut();
        return internal.delete(key);
    }

    pub(crate) fn devide(&mut self, node: &mut Self) -> bool {
        let (devided_node, supplied_node) = (self.as_internal_mut(), node.as_internal_mut());

        let length_sum = devided_node.length() + supplied_node.length();

        if (length_sum / 2) <= MIN_LEN {
            // return Failure
            return false;
        }

        let mut temp_keys: [MaybeUninit<_>; (CAPACITY * 2) + 1] = MaybeUninit::<K>::uninit_array();
        let mut temp_children: [MaybeUninit<_>; INTERNAL_CHILDREN_CAPACITY * 2] =
            MaybeUninit::<NodeRef<K, V, marker::LeafOrInternal>>::uninit_array();

        let devided_node_length = devided_node.length();
        temp_keys[0..devided_node_length - 1]
            .swap_with_slice(&mut devided_node.keys[0..devided_node_length - 1]);
        temp_keys[devided_node_length - 1].write(devided_node.get_largest_key());
        temp_children[0..devided_node_length]
            .swap_with_slice(&mut devided_node.children[0..devided_node_length]);

        let supplied_node_length = supplied_node.length();
        temp_keys[devided_node_length..(devided_node_length + supplied_node_length - 1)]
            .swap_with_slice(&mut supplied_node.keys[0..supplied_node_length - 1]);
        temp_children[devided_node_length..(devided_node_length + supplied_node_length)]
            .swap_with_slice(&mut supplied_node.children[0..supplied_node_length]);

        devided_node.keys[0..(length_sum / 2) - 1]
            .swap_with_slice(&mut temp_keys[0..(length_sum / 2) - 1]);
        devided_node.children[0..(length_sum / 2)]
            .swap_with_slice(&mut temp_children[0..(length_sum / 2)]);

        supplied_node.keys[0..length_sum - (length_sum / 2)]
            .swap_with_slice(&mut temp_keys[(length_sum / 2)..length_sum]);
        supplied_node.children[0..length_sum - (length_sum / 2)]
            .swap_with_slice(&mut temp_children[(length_sum / 2)..length_sum]);

        // lengthの修正
        devided_node.length = (length_sum / 2) as u16;
        supplied_node.length = (length_sum - (length_sum / 2)) as u16;

        // return Success
        return true;
    }

    pub(crate) fn marge(&mut self, leaf: &mut Self) {
        let (marged_node, marge_node) = (self.as_internal_mut(), leaf.as_internal_mut());

        let key = marged_node.get_largest_key();
        marged_node.keys[marged_node.length() - 1].write(key);

        if marge_node.length() == 1 {
            unsafe {
                marged_node.children[marged_node.length()]
                    .write(marge_node.children[0].assume_init_read());
            }
            marged_node.length += 1;
        } else {
            for idx in 0..marge_node.length() {
                unsafe {
                    marged_node.keys[marged_node.length()]
                        .write(marge_node.keys[idx].assume_init_read());
                    marged_node.children[marged_node.length()]
                        .write(marge_node.children[idx].assume_init_read());
                }
                marged_node.length += 1;
            }
        }
    }

    pub(crate) fn get_largest_key(&self) -> K {
        let internal = self.as_internal();
        internal.get_largest_key()
    }
}
impl<'a, K: Ord + Debug, V: Debug> NodeRef<K, V, marker::Leaf> {
    pub(crate) fn delete(&mut self, key: &K) -> DeleteBehavior<K, V> {
        let leaf = unsafe { self.node.ptr.as_mut() };
        return leaf.delete(key);
    }

    pub(crate) fn marge(&mut self, leaf: &mut Self) {
        let (marged_node, marge_node) = unsafe { (self.node.ptr.as_mut(), leaf.node.ptr.as_mut()) };
        for idx in 0..marge_node.length() {
            unsafe {
                marged_node.keys[marged_node.length()]
                    .write(marge_node.keys[idx].assume_init_read());
                marged_node.vals[marged_node.length()]
                    .write(marge_node.vals[idx].assume_init_read());
            }
            marged_node.length += 1;
        }

        marged_node.next_leaf = marge_node.next_leaf.take();
    }

    pub(crate) fn devide(&mut self, leaf: &mut Self) -> bool {
        let (devided_node, supplied_node) =
            unsafe { (self.node.ptr.as_mut(), leaf.node.ptr.as_mut()) };

        let length_sum = devided_node.length() + supplied_node.length();

        if (length_sum / 2) <= MIN_LEN {
            // return Failure
            return false;
        }

        let mut temp_keys: [MaybeUninit<_>; CAPACITY * 2] = MaybeUninit::<K>::uninit_array();
        let mut temp_vals: [MaybeUninit<_>; CAPACITY * 2] = MaybeUninit::<V>::uninit_array();

        let devided_node_length = devided_node.length();
        temp_keys[0..devided_node_length]
            .swap_with_slice(&mut devided_node.keys[0..devided_node_length]);
        temp_vals[0..devided_node_length]
            .swap_with_slice(&mut devided_node.vals[0..devided_node_length]);
        let supplied_node_length = supplied_node.length();
        temp_keys[devided_node_length..(devided_node_length + supplied_node_length)]
            .swap_with_slice(&mut supplied_node.keys[0..supplied_node_length]);
        temp_vals[devided_node_length..(devided_node_length + supplied_node_length)]
            .swap_with_slice(&mut supplied_node.vals[0..supplied_node_length]);

        devided_node.keys[0..(length_sum / 2)].swap_with_slice(&mut temp_keys[0..(length_sum / 2)]);
        devided_node.vals[0..(length_sum / 2)].swap_with_slice(&mut temp_vals[0..(length_sum / 2)]);

        supplied_node.keys[0..(length_sum - (length_sum / 2))]
            .swap_with_slice(&mut temp_keys[(length_sum / 2)..length_sum]);
        supplied_node.vals[0..(length_sum - (length_sum / 2))]
            .swap_with_slice(&mut temp_vals[(length_sum / 2)..length_sum]);

        // lengthの修正
        devided_node.length = (length_sum / 2) as u16;
        supplied_node.length = (length_sum - (length_sum / 2)) as u16;

        // return Success
        return true;
    }

    pub(crate) fn get_largest_key(&self) -> K {
        unsafe { self.node.ptr.as_ref().get_largest_key() }
    }
}

#[derive(Debug)]
pub(crate) enum DeleteBehavior<K: Debug, V: Debug> {
    RaiseChild(NodeRef<K, V, marker::LeafOrInternal>, Option<(usize, V)>),
    Success(Option<(usize, V)>),
}

impl<'a, K: Ord + Debug, V: Debug> InternalNode<K, V> {
    pub(crate) fn delete(&mut self, key: &K) -> DeleteBehavior<K, V> {
        let (child_idx, child_length, val) = {
            let (child_idx, ret) = self.delete_aux(key);

            if let None = ret {
                return DeleteBehavior::Success(None);
            };

            let (child_length, val) = ret.unwrap();
            (child_idx, child_length, val)
        };

        if self.length() <= 1 {
            let raised_node = unsafe { self.children[child_idx].assume_init_read() };
            return DeleteBehavior::RaiseChild(raised_node, Some((self.length(), val)));
        };

        // Check necessity balancing
        if child_length < MIN_LEN {
            let mut devide_or_marge =
                |idx_of_balanced_node: usize, idx_of_delete_execed_node: usize| unsafe {
                    let mut delete_execed_node =
                        self.children[idx_of_delete_execed_node].assume_init_read();
                    let balanced_node = self.children[idx_of_balanced_node].assume_init_mut();

                    let is_success = balanced_node.devide(&mut delete_execed_node);
                    if is_success {
                        let balanced_node_key = self.children[idx_of_balanced_node]
                            .assume_init_ref()
                            .get_largest_key();
                        self.keys[idx_of_balanced_node].write(balanced_node_key);
                        self.children[idx_of_delete_execed_node].write(delete_execed_node);
                    } else {
                        // try marge()
                        balanced_node.marge(&mut delete_execed_node);
                        self.length -= 1;
                        if CAPACITY != (self.length()) {
                            for idx in idx_of_delete_execed_node..self.length() {
                                let key_idx = idx - 1;
                                self.keys.swap(key_idx, key_idx + 1);
                                self.children.swap(idx, idx + 1);
                            }
                        }
                    }
                };

            if child_idx == 0 {
                devide_or_marge(0, 1);
            } else {
                devide_or_marge(child_idx - 1, child_idx);
            }
        }

        DeleteBehavior::Success(Some((self.length(), val)))
    }

    pub(crate) fn delete_aux(&mut self, key: &K) -> (usize, Option<(usize, V)>) {
        if self.length() == 0 {
            let ret = unsafe { self.children[0].assume_init_mut().delete(key) };
            return (0, ret);
        }

        for idx in 0..self.length() - 1 {
            // 挿入位置を決定する。
            let next = unsafe { self.keys[idx].assume_init_read() };
            if key <= &next {
                let ret = unsafe { self.children[idx].assume_init_mut().delete(key) };
                return (idx, ret);
            };
        }
        // ノードが保持するどのkeyよりも大きいkeyとして取り扱う。
        let idx = self.length() - 1;
        let ret = unsafe { self.children[idx].assume_init_mut().delete(key) };
        return (idx, ret);
    }

    pub(crate) fn get_largest_key(&self) -> K {
        unsafe {
            self.children[self.length() - 1]
                .assume_init_ref()
                .get_largest_key()
        }
    }
}

impl<'a, K: Ord + Debug, V: Debug> LeafNode<K, V> {
    pub(crate) fn delete(&mut self, key: &K) -> DeleteBehavior<K, V> {
        // keyが存在するか確認
        let idx = {
            let matching_key = |x: &MaybeUninit<K>| unsafe { x.assume_init_ref() == key };
            let idx = self.keys[0..self.length()].iter().position(matching_key);
            if let None = idx {
                return DeleteBehavior::Success(None);
            }
            idx.unwrap()
        };
        let ret = unsafe { self.vals[idx].assume_init_read() };

        // 削除処理
        self.keys[idx] = MaybeUninit::uninit();
        self.vals[idx] = MaybeUninit::uninit();
        if idx + 1 != (self.length()) {
            for idx in idx..self.length() - 1 {
                self.keys.swap(idx, idx + 1);
                self.vals.swap(idx, idx + 1);
            }
        }
        self.length -= 1;
        DeleteBehavior::Success(Some((self.length(), ret)))
    }

    pub(crate) fn get_largest_key(&self) -> K {
        unsafe { self.keys[self.length() - 1].assume_init_read() }
    }
}
