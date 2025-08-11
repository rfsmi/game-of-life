enum Tree {
    Leaf {
        alive: bool,
    },
    Branch {
        level: usize,
        subtree: [TreeRef; 4],
        population: usize,
    },
}

impl Tree {
    fn get_level(&self) -> usize {
        match *self {
            Tree::Leaf { .. } => 0,
            Tree::Branch { level, .. } => level,
        }
    }
}

trait Population {
    fn get_population(self) -> usize;
}

impl<T, P> Population for T
where
    T: IntoIterator<Item = P>,
    P: Population,
{
    fn get_population(self) -> usize {
        self.into_iter().map(|i| i.get_population()).sum()
    }
}

impl Population for &Tree {
    fn get_population(self) -> usize {
        match self {
            Tree::Leaf { alive: false } => 0,
            Tree::Leaf { alive: true } => 1,
            Tree::Branch { population, .. } => *population,
        }
    }
}

#[derive(Clone, Copy)]
struct TreeRef(usize);

struct Universe {
    nodes: Vec<Tree>,
}

impl Universe {
    #[inline]
    fn deref<'a>(&'a self) -> impl Fn(TreeRef) -> &'a Tree {
        |TreeRef(i): TreeRef| &self.nodes[i]
    }

    fn create_leaf(&mut self, alive: bool) -> TreeRef {
        self.nodes.push(Tree::Leaf { alive });
        TreeRef(self.nodes.len())
    }

    fn create_branch(&mut self, subtree: [TreeRef; 4]) -> TreeRef {
        let tree = Tree::Branch {
            population: subtree.map(self.deref()).get_population(),
            level: self.deref()(subtree[0]).get_level() + 1,
            subtree,
        };
        self.nodes.push(tree);
        TreeRef(self.nodes.len())
    }

    fn set_bit(&mut self, tr: TreeRef, (x, y): (isize, isize)) -> TreeRef {
        match self.get_tree(tr).level {}
        match (x, y) {
            (..0, 0..) => (),
            (_) => (),
        };
        todo!()
    }
}
