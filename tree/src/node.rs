use std::collections::HashMap;

pub struct Node<'a, V>
where
    V: Default,
{
    pub value: V,
    pub parent: Option<usize>,
    pub children: HashMap<&'a str, usize>,
}

impl<'a, V> Node<'a, V>
where
    V: Default,
{
    pub(crate) fn new(parent: Option<usize>, value: V) -> Self {
        Node {
            value,
            parent,
            children: HashMap::new(),
        }
    }

    pub(crate) fn display<F>(&self, mut indent: usize, arena: &Vec<Option<Self>>, f: F)
    where
        F: Fn(&Self) -> Option<String> + Copy,
    {
        indent += 1;
        let mut children: Vec<(_, _)> = self.children.iter().collect();
        children.sort_by(|a, b| a.0.cmp(b.0));
        for (name, child_index) in children {
            let child = arena[*child_index].as_ref().unwrap();

            // indenting from https://stackoverflow.com/a/42273813
            match f(child) {
                None => println!("{:indent$}{}", "", name, indent = indent),
                Some(t) => println!("{:>20}  {:indent$}{}", t, "", name, indent = indent * 2),
            }

            child.display(indent, arena, f);
        }
    }
}
