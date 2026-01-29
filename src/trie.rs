use std::collections::BTreeSet;

pub(crate) const TRIE_ASCII_SIZE: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrieNode<const N: usize> {
    end: bool, // represents whether the node is the final node for an input
    children: [Option<Box<TrieNode<N>>>; N],
}

impl<const N: usize> TrieNode<N> {
    pub(crate) fn new() -> Self {
        let children = std::array::from_fn(|_| None);
        TrieNode {
            end: false,
            children,
        }
    }

    pub(crate) fn create_end_node() -> Self {
        let children = std::array::from_fn(|_| None);
        TrieNode {
            end: true,
            children,
        }
    }

    pub(crate) fn set_end_node(&mut self) {
        self.end = true;
    }

    // pub(crate) fn contains(&self, key: &str) -> bool {
    //     let mut x = self;
    //     for c in key.chars() {
    //         let i = (c as u8) as usize;
    //         x = match x.children[i].as_deref() {
    //             Some(child) => child,
    //             None => return false,
    //         }
    //     }
    //     true
    // }

    pub(crate) fn insert(&mut self, key: &str) {
        let mut x = self;
        let count = key.chars().count();
        for (i, c) in key.chars().enumerate() {
            let idx = (c as u8) as usize;
            if idx > TRIE_ASCII_SIZE {
                return;
            }

            if x.children[idx].is_none() {
                if i == count - 1 {
                    x.children[idx] = Some(Box::new(TrieNode::create_end_node()))
                } else {
                    x.children[idx] = Some(Box::new(TrieNode::new()))
                }
            } else if i == count - 1
                && let Some(child) = x.children[idx].as_deref_mut()
            {
                child.set_end_node();
            }
            // unwrap as by prev if this value is always set
            x = x.children[idx].as_deref_mut().unwrap()
        }
    }

    pub(crate) fn auto_complete(&self, prefix: &str) -> Option<Vec<String>> {
        let mut x = self;
        let mut res = vec![];
        // set x to last node of prefix
        for c in prefix.chars() {
            let i = (c as u8) as usize;
            if i > TRIE_ASCII_SIZE {
                return None;
            }
            x = x.children[i].as_deref()?
        }

        // find all posiible string ends
        let postfixs = x.dfs();

        for postfix in postfixs {
            let str = format!("{prefix}{postfix}");
            res.push(str);
        }

        Some(res)
    }

    fn dfs(&self) -> BTreeSet<String> {
        let mut q = vec![];
        let mut res = BTreeSet::new();
        let x = self;

        let buf = String::new();

        q.push((x, buf));

        while let Some((node, curr_str)) = q.pop() {
            for (i, child) in node.children.iter().enumerate() {
                match child {
                    None => continue,
                    Some(n) => {
                        let c = (i as u8) as char;
                        let new_str = format!("{curr_str}{c}");
                        q.push((n.as_ref(), new_str));
                    }
                }
            }
            if node.end {
                res.insert(curr_str.clone());
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::TrieNode;

    #[test]
    fn auto_complete() {
        let mut trie: TrieNode<TRIE_ASCII_SIZE> = TrieNode::new();
        trie.insert("echo");
        trie.insert("exit");

        assert_eq!(
            trie.auto_complete("e"),
            Some(vec!["echo".into(), "exit".into()])
        );
        assert_eq!(trie.auto_complete("ech"), Some(vec!["echo".into()]));
    }

    #[test]
    fn auto_complete_overlap() {
        let mut trie: TrieNode<TRIE_ASCII_SIZE> = TrieNode::new();
        trie.insert("xyz_dog");
        trie.insert("xyz_dog_owl");
        trie.insert("xyz_dog_owl_cow");

        assert_eq!(
            trie.auto_complete("xyz_"),
            Some(vec![
                "xyz_dog".into(),
                "xyz_dog_owl".into(),
                "xyz_dog_owl_cow".into()
            ])
        );
    }
}
