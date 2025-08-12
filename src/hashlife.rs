use crate::state::State;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TreeRef(usize);

#[derive(Default)]
struct Universe {
    nodes: Vec<Tree>,
}

impl Universe {
    fn empty_tree(&mut self, level: usize) -> TreeRef {
        if level == 0 {
            return self.create_leaf(false);
        }
        let tr = self.empty_tree(level - 1);
        self.create_branch([tr, tr, tr, tr])
    }

    fn create_leaf(&mut self, alive: bool) -> TreeRef {
        self.nodes.push(Tree::Leaf { alive });
        TreeRef(self.nodes.len() - 1)
    }

    fn create_branch(&mut self, subtree: [TreeRef; 4]) -> TreeRef {
        let tree = Tree::Branch {
            population: subtree.map(self.deref()).get_population(),
            level: self.deref()(subtree[0]).get_level() + 1,
            subtree,
        };
        self.nodes.push(tree);
        TreeRef(self.nodes.len() - 1)
    }

    fn to_state(&self, tr: TreeRef) -> State {
        let mut state = State::default();
        match self.deref()(tr) {
            Tree::Leaf { alive: false } => (),
            Tree::Leaf { alive: true } => state.set_bit((0, 0)),
            Tree::Branch { level, .. } => {
                let w = 1 << (level - 1);
                for y in -w..w {
                    for x in -w..w {
                        if self.get_bit(tr, (y, x)) {
                            state.set_bit((y, x));
                        }
                    }
                }
            }
        }
        state
    }
}

impl Universe {
    #[inline]
    fn deref<'a>(&'a self) -> impl Fn(TreeRef) -> &'a Tree {
        |TreeRef(i)| &self.nodes[i]
    }

    fn expand_universe(&mut self, tr: TreeRef) -> TreeRef {
        let &Tree::Branch {
            level,
            subtree: [nw, ne, sw, se],
            ..
        } = self.deref()(tr)
        else {
            return self.create_branch([tr, tr, tr, tr]);
        };
        let border = self.empty_tree(level - 1);
        let subtree = [
            self.create_branch([border, border, border, nw]),
            self.create_branch([border, border, ne, border]),
            self.create_branch([border, sw, border, border]),
            self.create_branch([se, border, border, border]),
        ];
        self.create_branch(subtree)
    }

    fn set_bit(&mut self, tr: TreeRef, (y, x): (isize, isize)) -> TreeRef {
        let &Tree::Branch {
            level,
            subtree: [nw, ne, sw, se],
            ..
        } = self.deref()(tr)
        else {
            return self.create_leaf(true);
        };
        // Center on our child, which means moving half of its width.
        let offset = 1 << (level - 2);
        let subtree = match (y, x) {
            (..0, ..0) => [self.set_bit(nw, (y + offset, x + offset)), ne, sw, se],
            (..0, 0..) => [nw, self.set_bit(ne, (y + offset, x - offset)), sw, se],
            (0.., ..0) => [nw, ne, self.set_bit(sw, (y - offset, x + offset)), se],
            (0.., 0..) => [nw, ne, sw, self.set_bit(se, (y - offset, x - offset))],
        };
        self.create_branch(subtree)
    }

    fn get_bit(&self, tr: TreeRef, (y, x): (isize, isize)) -> bool {
        match *self.deref()(tr) {
            Tree::Leaf { alive } => alive,
            Tree::Branch {
                level,
                subtree: [nw, ne, sw, se],
                ..
            } => {
                let offset = 1 << (level - 2);
                match (y, x) {
                    (..0, ..0) => self.get_bit(nw, (y + offset, x + offset)),
                    (..0, 0..) => self.get_bit(ne, (y + offset, x - offset)),
                    (0.., ..0) => self.get_bit(sw, (y - offset, x + offset)),
                    (0.., 0..) => self.get_bit(se, (y - offset, x - offset)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_set_bit() {
        let mut universe = Universe::default();
        let tr = universe.empty_tree(4);
        let tr = universe.set_bit(tr, (0, 0));
        assert_eq!(universe.to_state(tr), State::from_str("o").unwrap());
    }
}
