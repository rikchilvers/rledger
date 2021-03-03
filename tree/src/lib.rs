use std::collections::HashMap;

struct Tree<'a, V>
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

    pub fn index_of_node_at_path(&self, path: &mut [&'a str]) -> Option<usize> {
        match path.split_last_mut() {
            None => return Some(self.root),
            Some((_, rest)) => match self.index_of_node_at_path(rest) {
                None => return None,
                Some(index) => return Some(index),
            },
        }
    }

    /// If the path did not exist, return None
    pub fn get_node_at_path(&mut self, path: &mut [&'a str]) -> Option<&Node<'a, V>> {
        match self.index_of_node_at_path(path) {
            None => return None,
            Some(index) => return self.arena[index].as_ref(),
        }
    }

    /// If the path did not exist, return None
    pub fn get_node_at_path_mut(&mut self, path: &mut [&'a str]) -> Option<&mut Node<'a, V>> {
        match self.index_of_node_at_path(path) {
            None => return None,
            Some(index) => return self.arena[index].as_mut(),
        }
    }

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

    pub fn walk_descendants<F>(&mut self, root: usize, mut f: F)
    where
        F: FnMut(&mut Node<'a, V>) + Copy,
    {
        // TODO I think we need to use the visitor pattern
        // https://sachanganesh.com/programming/graph-tree-traversals-in-rust/
        unimplemented!();
        /*
        match self.arena.get_mut(root) {
            None => return,
            Some(node) => match node {
                None => return,
                Some(node) => {
                    for index in node.children.values() {
                        self.walk_descendants(*index, f);
                        println!("hello");
                    }
                    f(node);
                }
            },
        }
        match self.arena[root].as_mut() {
            None => return,
            Some(node) => {
                f(node);

                for child_index in node.children.values() {
                    self.walk_descendants(*child_index, f);
                }
            }
        }
        */
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

    pub fn remove_node_at_path(&mut self, path: Vec<&'a str>) -> bool {
        unimplemented!();
    }

    pub fn remove_path(&mut self, path: Vec<&'a str>) -> bool {
        unimplemented!();
    }
}

struct Node<'a, V>
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

    pub fn get_parent(&self) -> &Option<usize> {
        &self.parent
    }

    pub fn get_children(&self) -> impl std::iter::Iterator<Item = (&&str, &usize)> {
        self.children.iter()
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

    fn walk_descendants<F>(&self, arena: &Vec<Option<Self>>, f: F)
    where
        F: FnMut(&mut Self) + Copy,
    {
        unimplemented!()
    }
}

/*
fn main() {
    let mut tree: Tree<'_, usize> = Tree::new();
    let mut path = vec!["a", "b", "c"];

    let c_index = tree.add_path(&mut path);
    println!("index of c is: {}", c_index);

    tree.walk_ancestors(c_index, |n| n.value += 1);

    let c = tree.arena[c_index].as_ref().unwrap();
    println!("parent of c is: {:?}", c.parent);

    // now we'll add c' to b
    let mut path = vec!["a", "b", "c'"];

    let c_prime_index = tree.add_path(&mut path);
    println!("index of c' is: {}", c_prime_index);

    let c_prime = tree.arena[c_prime_index].as_ref().unwrap();
    println!("parent of c' is: {:?}", c_prime.parent);

    let mut path = vec!["a", "b", "c", "d"];
    tree.add_path(&mut path);

    tree.display(&None, |node| Some(format!("{}", node.value)));

    let mut path = vec!["a", "b", "c'", "d'"];
    let d_prime_idx = tree.add_path(&mut path);

    tree.display(&None, |_| None);

    let d_prime = tree.get_node_at_path(&mut path).unwrap();
    println!("d_prime is {}", d_prime.value);
    let mut d_prime = tree.get_node_at_path_mut(&mut path).unwrap();
    d_prime.value = 42;
    let d_prime = tree.get_node_at_path(&mut path).unwrap();
    println!("d_prime is {}", d_prime.value);

    tree.walk_ancestors(d_prime_idx, |_| {});
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut root = TreeNode::new(0);

        let mut path = vec!["a", "b", "c"];
        path.reverse();

        let three = root.find_or_create_node(Some(3), path);
        assert_eq!(three.value, 3);
    }
}
