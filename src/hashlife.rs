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

    fn expand_universe(&mut self, [nw, ne, sw, se]: [TreeRef; 4]) -> TreeRef {
        let level = self.deref()(nw).get_level();
        let border = self.empty_tree(level);
        let subtree = [
            self.create_branch([border, border, border, nw]),
            self.create_branch([border, border, ne, border]),
            self.create_branch([border, sw, border, border]),
            self.create_branch([se, border, border, border]),
        ];
        self.create_branch(subtree)
    }

    fn one_gen(&mut self, bitmask: u16) -> TreeRef {
        let alive = bitmask & 0b0000_0010_0000 != 0;
        let neighbours = (bitmask & 0b0111_0101_0111).count_ones();
        self.create_leaf(match (alive, neighbours) {
            (true, 2 | 3) | (false, 3) => true,
            _ => false,
        })
    }

    fn two_gen(&mut self, tr: TreeRef) -> TreeRef {
        let mut bitmask = 0;
        for y in -2..2 {
            for x in -2..2 {
                bitmask <<= 1;
                if self.get_bit(tr, (y, x)) {
                    bitmask += 1;
                }
            }
        }
        let subtree = [
            self.one_gen(bitmask >> 5),
            self.one_gen(bitmask >> 4),
            self.one_gen(bitmask >> 1),
            self.one_gen(bitmask >> 0),
        ];
        self.create_branch(subtree)
    }
}

enum FindResult {
    Found {
        alive: bool,
    },
    Deeper {
        p: (isize, isize),
        idx: usize,
        subtree: [TreeRef; 4],
    },
}

impl Universe {
    #[inline]
    fn find(&self, tr: TreeRef, (y, x): (isize, isize)) -> FindResult {
        match *self.deref()(tr) {
            Tree::Leaf { alive } => FindResult::Found { alive },
            Tree::Branch { level, subtree, .. } => {
                let offset = (1 << level) / 4;
                let (idx, p) = match (y, x) {
                    (..0, ..0) => (0, (y + offset, x + offset)),
                    (..0, 0..) => (1, (y + offset, x - offset)),
                    (0.., ..0) => (2, (y - offset, x + offset)),
                    (0.., 0..) => (3, (y - offset, x - offset)),
                };
                FindResult::Deeper { p, subtree, idx }
            }
        }
    }

    fn set_bit(&mut self, tr: TreeRef, p: (isize, isize)) -> TreeRef {
        match self.find(tr, p) {
            FindResult::Found { .. } => self.create_leaf(true),
            FindResult::Deeper {
                p,
                idx,
                mut subtree,
            } => {
                subtree[idx] = self.set_bit(subtree[idx], p);
                self.create_branch(subtree)
            }
        }
    }

    fn get_bit(&self, tr: TreeRef, p: (isize, isize)) -> bool {
        match self.find(tr, p) {
            FindResult::Found { alive } => alive,
            FindResult::Deeper { p, idx, subtree } => self.get_bit(subtree[idx], p),
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
        let tr = universe.empty_tree(5);
        let tr = universe.set_bit(tr, (0, 0));
        let tr = universe.set_bit(tr, (0, 3));
        let tr = universe.set_bit(tr, (0, 4));
        let tr = universe.set_bit(tr, (0, 8));
        let tr = universe.set_bit(tr, (0, 7));
        let tr = universe.set_bit(tr, (0, 9));

        let tr = universe.set_bit(tr, (0, -1));
        let tr = universe.set_bit(tr, (3, -1));
        let tr = universe.set_bit(tr, (4, -1));
        assert_eq!(
            universe.to_state(tr).normalize(),
            State::from_str(
                "
                oo  oo  ooo
                
                
                o
                o
                "
            )
            .unwrap()
        );
    }

    #[test]
    fn test_expand_universe() {
        let mut universe = Universe::default();
        let alive = universe.create_leaf(true);
        let dead = universe.create_leaf(false);
        let tr = universe.expand_universe([dead, alive, alive, dead]);
        assert_eq!(
            universe.to_state(tr).normalize(),
            State::from_str(
                "
                 o
                o
                "
            )
            .unwrap()
        );
    }
}
