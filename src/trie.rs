use std::collections::BTreeSet;

pub(crate) const TRIE_ASCII_SIZE: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrieNode<const N: usize> {
    children: [Option<Box<TrieNode<N>>>; N],
}

impl<const N: usize> TrieNode<N> {
    pub(crate) fn new() -> Self {
        let children = std::array::from_fn(|_| None);
        TrieNode { children }
    }

    pub(crate) fn contains(&self, key: &str) -> bool {
        let mut x = self;
        for c in key.chars() {
            let i = (c as u8) as usize;
            x = match x.children[i].as_deref() {
                Some(child) => child,
                None => return false,
            }
        }
        true
    }

    pub(crate) fn insert(&mut self, key: &str) {
        let mut x = self;
        for c in key.chars() {
            let i = (c as u8) as usize;
            if i > TRIE_ASCII_SIZE {
                return;
            }
            if x.children[i].is_none() {
                x.children[i] = Some(Box::new(TrieNode::new()))
            }
            // unwrap as by prev if this value is always set
            x = x.children[i].as_deref_mut().unwrap()
        }
    }

    fn delete(&mut self, _key: &str) {
        todo!();
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
            let mut leaf_node = true;
            for (i, child) in node.children.iter().enumerate() {
                match child {
                    None => continue,
                    Some(n) => {
                        let c = (i as u8) as char;
                        let new_str = format!("{curr_str}{c}");
                        leaf_node = false;
                        q.push((n.as_ref(), new_str));
                    }
                }
            }
            if leaf_node {
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

        assert!(trie.contains("echo"));
        assert!(trie.contains("exit"));

        assert_eq!(
            trie.auto_complete("e"),
            Some(vec!["echo".into(), "exit".into()])
        );
        assert_eq!(trie.auto_complete("ech"), Some(vec!["echo".into()]));
    }
}
