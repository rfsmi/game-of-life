use std::collections::HashMap;

use itertools::Itertools;

use crate::state::State;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
    next_gen: HashMap<TreeRef, TreeRef>,
    interned_nodes: HashMap<Tree, TreeRef>,
}

impl Universe {
    fn empty_tree(&mut self, level: usize) -> TreeRef {
        if level == 0 {
            return self.create_leaf(false);
        }
        let tr = self.empty_tree(level - 1);
        self.create_branch([tr, tr, tr, tr])
    }

    fn canonicalise(&mut self, tree: Tree) -> TreeRef {
        *self.interned_nodes.entry(tree).or_insert_with_key(|&tree| {
            let tr = TreeRef(self.nodes.len());
            self.nodes.push(tree);
            tr
        })
    }

    fn create_leaf(&mut self, alive: bool) -> TreeRef {
        self.canonicalise(Tree::Leaf { alive })
    }

    fn create_branch(&mut self, subtree: [TreeRef; 4]) -> TreeRef {
        let tree = Tree::Branch {
            population: subtree.map(self.deref()).get_population(),
            level: self.deref()(subtree[0]).get_level() + 1,
            subtree,
        };
        self.canonicalise(tree)
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

    fn from_state(&mut self, state: State) -> TreeRef {
        let mut r = 1;
        let mut tr = self.empty_tree(1);
        for (y, x) in state.cells {
            while y.abs() >= r || x.abs() >= r {
                tr = self.expand_universe(tr);
                r *= 2;
            }
            tr = self.set_bit(tr, (y, x));
        }
        tr
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
            panic!();
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

    fn l2_gen(&mut self, bitmask: u16) -> TreeRef {
        fn alive(bitmask: u16) -> bool {
            let alive = bitmask & 0b0000_0010_0000 != 0;
            let neighbours = (bitmask & 0b0111_0101_0111).count_ones();
            match (alive, neighbours) {
                (true, 2 | 3) | (false, 3) => true,
                _ => false,
            }
        }
        let subtree = [
            self.create_leaf(alive(bitmask >> 5)),
            self.create_leaf(alive(bitmask >> 4)),
            self.create_leaf(alive(bitmask >> 1)),
            self.create_leaf(alive(bitmask >> 0)),
        ];
        self.create_branch(subtree)
    }

    fn make_l2_bitmask(&self, tr: TreeRef) -> u16 {
        let mut bitmask = 0;
        for y in -2..2 {
            for x in -2..2 {
                bitmask <<= 1;
                if self.get_bit(tr, (y, x)) {
                    bitmask += 1;
                }
            }
        }
        bitmask
    }

    fn get_node(&self, depth: usize, tr: TreeRef, p: (isize, isize)) -> TreeRef {
        if depth == 0 {
            return tr;
        }
        let &Tree::Branch { subtree, .. } = self.deref()(tr) else {
            panic!();
        };
        let (idx, p) = self.choose_subtree(depth, p);
        self.get_node(depth - 1, subtree[idx], p)
    }

    fn translated_subtree(
        &mut self,
        tr: TreeRef,
        current: usize,
        depth: usize,
        (y, x): (isize, isize),
    ) -> TreeRef {
        let ps = match depth - current {
            0 => return self.get_node(depth, tr, (y, x)),
            1 => [(y - 1, x - 1), (y - 1, x), (y, x - 1), (y, x)],
            level => {
                let offset = 1 << (level - 2);
                [
                    (y - offset, x - offset),
                    (y - offset, x + offset),
                    (y + offset, x - offset),
                    (y + offset, x + offset),
                ]
            }
        };
        let subtree = ps.map(|p| self.translated_subtree(tr, current + 1, depth, p));
        self.create_branch(subtree)
    }

    fn next_generation(&mut self, tr: TreeRef) -> TreeRef {
        if let Some(&tr) = self.next_gen.get(&tr) {
            return tr;
        }
        let &Tree::Branch {
            level,
            subtree,
            population,
        } = self.deref()(tr)
        else {
            panic!();
        };
        if population == 0 {
            return subtree[0];
        }
        if level == 2 {
            let bitmask = self.make_l2_bitmask(tr);
            return self.l2_gen(bitmask);
        }
        let subtree = [
            self.translated_subtree(tr, 1, 3, (-1, -1)),
            self.translated_subtree(tr, 1, 3, (-1, 1)),
            self.translated_subtree(tr, 1, 3, (1, -1)),
            self.translated_subtree(tr, 1, 3, (1, 1)),
        ]
        .map(|tr| self.next_generation(tr));
        let next_tr = self.create_branch(subtree);
        self.next_gen.insert(tr, next_tr);
        next_tr
    }
}

impl Universe {
    fn choose_subtree(&self, depth: usize, (y, x): (isize, isize)) -> (usize, (isize, isize)) {
        let offset = (1 << depth) / 4;
        match (y, x) {
            (..0, ..0) => (0, (y + offset, x + offset)),
            (..0, 0..) => (1, (y + offset, x - offset)),
            (0.., ..0) => (2, (y - offset, x + offset)),
            (0.., 0..) => (3, (y - offset, x - offset)),
        }
    }

    fn set_bit(&mut self, tr: TreeRef, p: (isize, isize)) -> TreeRef {
        let &tree = self.deref()(tr);
        match tree {
            Tree::Leaf { alive: false } => self.create_leaf(true),
            Tree::Leaf { alive: true } => tr,
            Tree::Branch {
                level, mut subtree, ..
            } => {
                let (idx, p) = self.choose_subtree(level, p);
                subtree[idx] = self.set_bit(subtree[idx], p);
                self.create_branch(subtree)
            }
        }
    }

    fn get_bit(&self, tr: TreeRef, p: (isize, isize)) -> bool {
        match *self.deref()(tr) {
            Tree::Leaf { alive } => alive,
            Tree::Branch { level, subtree, .. } => {
                let (idx, p) = self.choose_subtree(level, p);
                self.get_bit(subtree[idx], p)
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
    fn test_glider() {
        // Test a boat + glider combo
        let states = [
            "
   oo       o
   o o       o
    o      ooo",
            "
   oo
   o o     o o
    o       oo
            o",
            "
   oo 
   o o       o
    o      o o
            oo",
            "
   oo
   o o      o
    o        oo
            oo",
            "
   oo
   o o       o
    o         o
            ooo",
            "
   oo
   o o
    o       o o
             oo
             o",
        ];
        let mut universe = Universe::default();
        for (a, b) in states.into_iter().tuple_windows() {
            let a = universe.from_state(State::from_str(a).unwrap());
            let a = universe.expand_universe(a);
            let a_step = universe.next_generation(a);
            let b = State::from_str(b).unwrap();
            assert_eq!(universe.to_state(a_step).normalize(), b);
        }
    }
}
