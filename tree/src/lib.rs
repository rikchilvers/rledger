use std::collections::HashMap;

pub struct Tree<'a, V>
where
    V: Default,
{
    arena: Vec<Option<Node<'a, V>>>,
    root: usize,
}

impl<'a, V> Tree<'a, V>
where
    V: Default,
{
    pub fn new() -> Self {
        let mut tree = Tree {
            arena: Vec::new(),
            root: 0,
        };
        tree.arena.push(Some(Node {
            value: V::default(),
            parent: None,
            children: HashMap::new(),
        }));
        tree
    }

    /// Returns the index of the last path component
    /// Will add all intermidiate nodes as necessary
    /// These nodes will have the default value
    pub fn add_path(&mut self, path: &mut [&'a str]) -> usize {
        match path.split_last_mut() {
            None => {
                // Eventually, we have to return the root
                return self.root;
            }
            Some((component, path)) => {
                // println!("popped {}", component);

                // Will eventually end up at the root
                let parent_index = self.add_path(path);

                // This is where we'll insert the new node
                let node_index = self.arena.len();

                // FIXME this isn't ideal - we should probably use get_mut
                // but maybe we don't need to because we know we'll always get one?
                let parent = self.arena[parent_index].as_mut().unwrap();

                // TODO check if the parent has the component before blindly adding
                if let Some(child_index) = parent.children.get(component) {
                    // println!("\tfound child for {} on {}", component, parent_index);
                    return *child_index;
                }

                // println!("\tdid not find child for {} on {}", component, parent_index);

                parent.children.insert(component, node_index);

                // Add the new node
                let node = Node::new(Some(parent_index), V::default());
                self.arena.push(Some(node));

                // Return that node index
                node_index
            }
        }
    }

    /// Convenience method for adding a node and setting it's value
    pub fn add_value_at_path(&mut self, path: &mut [&'a str], value: V) {
        let index = self.add_path(path);
        let node = self.get_node_at_path_mut(path).unwrap();
        node.value = value;
    }

    /// If the path did not exist, return None
    pub fn get_node_at_path(&mut self, path: &mut [&'a str]) -> Option<&Node<'a, V>> {
        match index_of_node_at_path(&self.arena, path, self.root) {
            None => return None,
            Some(index) => return self.arena[index].as_ref(),
        }
    }

    /// If the path did not exist, return None
    pub fn get_node_at_path_mut(&mut self, path: &mut [&'a str]) -> Option<&mut Node<'a, V>> {
        match index_of_node_at_path(&self.arena, path, self.root) {
            None => None,
            Some(index) => self.arena[index].as_mut(),
        }
    }

    /// Applies `F` to root and all ancestors
    pub fn walk_ancestors<F>(&mut self, root: usize, mut f: F)
    where
        F: FnMut(&mut Node<'a, V>),
    {
        if root == self.root {
            return;
        }

        match self.arena[root].as_mut() {
            None => return,
            Some(node) => {
                f(node);

                if let Some(parent_index) = node.parent {
                    self.walk_ancestors(parent_index, f);
                }
            }
        }
    }

    /// Applies `F` to all descendants of root
    pub fn walk_descendants<F>(&mut self, root: usize, mut f: F)
    where
        F: FnMut(&mut Node<'a, V>) + Copy,
    {
        if let Some(indices) = self.child_indices(root) {
            for index in indices {
                let node = self.arena[index].as_mut().unwrap();
                f(node);
            }
        }
    }

    fn child_indices(&self, root: usize) -> Option<Vec<usize>> {
        match self.arena.get(root) {
            None => return None,
            Some(node) => match node {
                None => return None,
                Some(node) => {
                    if node.children.len() == 0 {
                        return None;
                    }

                    Some(node.children.values().fold(vec![], |mut acc, index| {
                        // Add the index of the child
                        acc.push(*index);

                        // Add the indicies of the grandchildren
                        if let Some(ref mut v) = self.child_indices(*index) {
                            acc.append(v);
                        }

                        acc
                    }))
                }
            },
        }
    }

    pub fn display<F>(&self, root: &Option<usize>, f: F)
    where
        F: Fn(&Node<'a, V>) -> Option<String> + Copy,
    {
        // Level of indent to start at
        let indent = 0;

        let root_node: &Node<'a, V> = match root {
            None => self.arena[self.root].as_ref().unwrap(),
            Some(root) => self.arena[*root].as_ref().unwrap(),
        };

        for (name, child_index) in root_node.children.iter() {
            let child = self.arena[*child_index].as_ref().unwrap();

            // indenting from https://stackoverflow.com/a/42273813
            match f(child) {
                None => println!("{:indent$}{}", "", name, indent = indent),
                Some(t) => println!("{:indent$}{}\t{}", "", name, t, indent = indent),
            }

            child.display(indent, &self.arena, f);
        }
    }
}

// We need this to be a free function so we don't have multiple borrows of the tree
fn index_of_node_at_path<'a, V>(arena: &Vec<Option<Node<'a, V>>>, path: &mut [&'a str], root: usize) -> Option<usize>
where
    V: Default,
{
    match path.split_last_mut() {
        // If we can't split the path anymore, we've got to the root
        None => return Some(root),

        Some((component, rest)) => match index_of_node_at_path(arena, rest, root) {
            None => return None,

            Some(index) => match arena.get(index) {
                None => return None,

                Some(node) => match node {
                    None => return None,
                    Some(node) => return node.children.get(component).map(|i| *i),
                },
            },
        },
    }
}

pub struct Node<'a, V>
where
    V: Default,
{
    pub value: V,
    parent: Option<usize>,
    children: HashMap<&'a str, usize>,
}

impl<'a, V> Node<'a, V>
where
    V: Default,
{
    fn new(parent: Option<usize>, value: V) -> Self {
        Node {
            value,
            parent,
            children: HashMap::new(),
        }
    }

    fn display<F>(&self, mut indent: usize, arena: &Vec<Option<Self>>, f: F)
    where
        F: Fn(&Self) -> Option<String> + Copy,
    {
        indent += 1;
        for (name, child_index) in self.children.iter() {
            let child = arena[*child_index].as_ref().unwrap();

            // indenting from https://stackoverflow.com/a/42273813
            match f(child) {
                None => println!("{:indent$}{}", "", name, indent = indent),
                Some(t) => println!("{:indent$}{}\t{}", "", name, t, indent = indent),
            }

            child.display(indent, arena, f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_adds_paths() {
        let mut tree: Tree<'_, usize> = Tree::new();
        let mut path = vec!["a", "b", "c"];
        let c_index = tree.add_path(&mut path);
        assert_eq!(3, c_index);
    }

    #[test]
    fn it_adds_values() {
        let mut tree: Tree<'_, usize> = Tree::new();
        let mut path = vec!["a", "b", "c"];
        tree.add_value_at_path(&mut path, 42);

        match tree.get_node_at_path(&mut ["a", "b"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 0),
        }

        match tree.get_node_at_path(&mut ["a", "b", "c"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 42),
        }
    }

    #[test]
    fn it_finds_the_index_of_a_path() {
        let mut tree: Tree<'_, usize> = Tree::new();
        let mut path = vec!["a", "b", "c"];
        tree.add_path(&mut path);

        assert_eq!(index_of_node_at_path(&tree.arena, &mut ["a"], tree.root), Some(1));
        assert_eq!(index_of_node_at_path(&tree.arena, &mut ["a", "b"], tree.root), Some(2));
        assert_eq!(
            index_of_node_at_path(&tree.arena, &mut ["a", "b", "c"], tree.root),
            Some(3)
        );
    }

    #[test]
    fn it_gets_a_node_at_a_path() {
        let mut tree: Tree<'_, usize> = Tree::new();
        let mut path = vec!["a", "b", "c"];
        tree.add_path(&mut path);

        match tree.get_node_at_path_mut(&mut ["a", "b"]) {
            None => panic!("failed to get node at path"),
            Some(node) => node.value = 42,
        }

        match tree.get_node_at_path(&mut ["a", "b"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 42),
        }
    }

    #[test]
    fn it_walks_ancestors() {
        let mut tree: Tree<'_, usize> = Tree::new();
        let mut path = vec!["a", "b", "c"];
        let c_index = tree.add_path(&mut path);

        match tree.get_node_at_path(&mut ["a", "b", "c"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 0),
        }

        match tree.get_node_at_path(&mut ["a", "b"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 0),
        }

        match tree.get_node_at_path(&mut ["a"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 0),
        }

        tree.walk_ancestors(c_index, |node| node.value += 1);

        match tree.get_node_at_path(&mut ["a"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 1),
        }

        match tree.get_node_at_path(&mut ["a", "b"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 1),
        }

        match tree.get_node_at_path(&mut ["a", "b", "c"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 1),
        }
    }

    #[test]
    fn it_walks_descendants() {
        let mut tree: Tree<'_, usize> = Tree::new();
        let mut path = vec!["a", "b", "c"];
        tree.add_path(&mut path);

        match tree.get_node_at_path(&mut ["a", "b", "c"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 0),
        }

        match tree.get_node_at_path(&mut ["a", "b"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 0),
        }

        match tree.get_node_at_path(&mut ["a"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 0),
        }

        tree.walk_descendants(0, |node| node.value += 1);

        match tree.get_node_at_path(&mut ["a"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 1),
        }

        match tree.get_node_at_path(&mut ["a", "b"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 1),
        }

        match tree.get_node_at_path(&mut ["a", "b", "c"]) {
            None => panic!("failed to get node at path"),
            Some(node) => assert_eq!(node.value, 1),
        }
    }
}
