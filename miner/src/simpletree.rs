pub trait Parenting {
    fn is_parent(&self, parent_id: &[u8]) -> bool;
    fn parent_hash(&self) -> &[u8];
    fn hash(&self) -> Vec<u8>;
    fn nonce(&self) -> u64;
}

#[derive(Debug, Default)]
pub struct TreeNode<T: Default + Parenting> {
    value: T,
    children: Vec<TreeNode<T>>,
}

impl<T: Default + Parenting> TreeNode<T> {
    /// Create a new tree node with the given value and no children
    pub fn new(value: T) -> Self {
        TreeNode {
            value,
            children: Vec::new(),
        }
    }

    /// Insert a new child node with the given value
    pub fn insert(&mut self, value: T) {
        self.children.push(TreeNode::new(value));
    }

    /// Remove all children nodes with the given value (recursively)
    #[allow(dead_code)]
    pub fn remove(&mut self, value: &T)
    where
        T: PartialEq,
    {
        self.children.retain_mut(|child| {
            child.remove(value);
            &child.value != value
        });
    }

    /// Calculate the depth of the tree from this node
    pub fn depth(&self) -> usize {
        let mut max_depth = 0;
        for child in &self.children {
            let child_depth = child.depth();
            if child_depth > max_depth {
                max_depth = child_depth;
            }
        }
        max_depth + 1
    }

    /// Perform a depth-first search looking for the parent
    pub fn look_for_parent(&mut self, parent_id: &[u8]) -> Option<&mut TreeNode<T>> {
        if self.value.is_parent(parent_id) {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(found) = child.look_for_parent(parent_id) {
                return Some(found);
            }
        }
        None
    }
    
    /// Finds the correct parent node for the given block and inserts it
    /// Returns true if the block was inserted, false otherwise
    pub fn find_and_insert<H>(&mut self, block: &T, _processed_blocks: &mut H) -> bool 
    where T: Clone {
        // First check if this node is the parent
        let parent_hash = block.parent_hash();
        let self_hash = self.value.hash();
        
        // If this node's hash matches the block's parent_hash, insert the block here
        if parent_hash.len() == self_hash.len() && 
           parent_hash.iter().zip(self_hash.iter()).all(|(a, b)| a == b) {
            // println!("DEBUG: Found exact parent match for block with hash {:?}", block.hash());
            self.insert(block.clone());
            return true;
        }
        
        // Otherwise, recursively check children
        for child in &mut self.children {
            if child.find_and_insert(block, _processed_blocks) {
                return true;
            }
        }
        
        false
    }

    /// Get the value of the node
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Get a reference to the children of the node
    pub fn children(&self) -> &Vec<TreeNode<T>> {
        &self.children
    }

    /// Get a mutable reference to the children of the node
    #[allow(dead_code)]
    pub fn children_mut(&mut self) -> &mut Vec<TreeNode<T>> {
        &mut self.children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Default, Debug)]
    struct Data {
        val: u32,
        parent_id: [u8; 1],
    }

    impl Data {
        fn new(val: u32, parent_id: [u8; 1]) -> Self {
            Data { val, parent_id }
        }
    }

    impl Parenting for Data {
        fn is_parent(&self, parent_id: &[u8]) -> bool {
            self.val as u8 == parent_id[0]
        }
        
        fn parent_hash(&self) -> &[u8] {
            &self.parent_id
        }
        
        fn hash(&self) -> Vec<u8> {
            vec![self.val as u8]
        }
        
        fn nonce(&self) -> u64 {
            self.val as u64
        }
    }

    #[test]
    fn test_tree_operations() {
        let mut root = TreeNode::new(Data::new(42, [0]));
        root.insert(Data::new(2, [42]));
        root.insert(Data::new(3, [42]));
        root.children_mut()[0].insert(Data::new(4, [2]));
        root.children_mut()[0].insert(Data::new(5, [2]));
        root.children_mut()[1].insert(Data::new(6, [3]));

        assert_eq!(root.depth(), 3);

        // Test look_for_parent
        assert_eq!(
            root.look_for_parent(&[3]).unwrap().value(),
            &Data::new(3, [42])
        );

        // Test remove
        root.remove(&Data::new(2, [42]));
        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].value(), &Data::new(3, [42]));
    }
}
