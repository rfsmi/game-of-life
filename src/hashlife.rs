use std::collections::HashMap;

use itertools::{chain, Itertools};

use crate::state::State;

#[derive(Clone)]
struct HashLifeState {
    universe: Universe,
    depth: usize,
    root: TreeRef,
}

impl HashLifeState {
    fn new() -> Self {
        let mut universe = Universe::default();
        let depth = 2;
        let root = universe.empty_tree(depth);
        Self {
            universe,
            depth,
            root,
        }
    }

    fn contains(&self, (y, x): (isize, isize)) -> bool {
        let w = 1 << (self.depth - 1);
        (-w..w).contains(&y) && (-w..w).contains(&x)
    }

    fn set_bit(&mut self, (y, x): (isize, isize)) {
        while !self.contains((y, x)) {
            self.root = self.universe.expand_universe(self.depth, self.root);
            self.depth += 1;
        }
        self.root = self.universe.set_bit(self.depth, self.root, (y, x));
    }

    fn get_bit(&self, p: (isize, isize)) -> bool {
        self.universe.get_bit(self.depth, self.root, p)
    }

    fn step(&mut self) {
        // We can only step if all the border nodes in the 4x4 square are empty.
        let empty = self.universe.empty_tree(self.depth - 2);
        if chain!(
            (-2..2).map(|x| (-2, x)),
            (-1..1).map(|y| (y, -2)),
            (-1..1).map(|y| (y, 1)),
            (-2..2).map(|x| (1, x)),
        )
        .any(|p| self.universe.get_node(2, self.root, p) != empty)
        {
            self.root = self.universe.expand_universe(self.depth, self.root);
            self.depth += 1;
        }
        self.root = self.universe.next_generation(self.depth, self.root);
        self.depth -= 1;
    }
}

impl From<State> for HashLifeState {
    fn from(value: State) -> Self {
        let mut state = HashLifeState::new();
        for p in value.cells {
            state.set_bit(p);
        }
        state
    }
}

impl From<HashLifeState> for State {
    fn from(hls: HashLifeState) -> Self {
        let mut state = State::default();
        match hls.universe.deref()(hls.root) {
            Tree::Leaf { alive: false } => (),
            Tree::Leaf { alive: true } => state.set_bit((0, 0)),
            Tree::Branch { .. } => {
                let w = 1 << (hls.depth - 1);
                for y in -w..w {
                    for x in -w..w {
                        if hls.get_bit((y, x)) {
                            state.set_bit((y, x));
                        }
                    }
                }
            }
        }
        state
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Tree {
    Leaf { alive: bool },
    Branch { subtree: [TreeRef; 4] },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TreeRef(usize);

#[derive(Default, Clone)]
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
        self.canonicalise(Tree::Branch { subtree })
    }
}

impl Universe {
    #[inline]
    fn deref<'a>(&'a self) -> impl Fn(TreeRef) -> &'a Tree {
        |TreeRef(i)| &self.nodes[i]
    }

    fn expand_universe(&mut self, level: usize, tr: TreeRef) -> TreeRef {
        let &Tree::Branch {
            subtree: [nw, ne, sw, se],
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
                if self.get_bit(2, tr, (y, x)) {
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
        let &Tree::Branch { subtree } = self.deref()(tr) else {
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

    fn next_generation(&mut self, depth: usize, tr: TreeRef) -> TreeRef {
        if let Some(&tr) = self.next_gen.get(&tr) {
            return tr;
        }
        if depth == 2 {
            let bitmask = self.make_l2_bitmask(tr);
            return self.l2_gen(bitmask);
        }
        let subtree = [
            self.translated_subtree(tr, 1, 3, (-1, -1)),
            self.translated_subtree(tr, 1, 3, (-1, 1)),
            self.translated_subtree(tr, 1, 3, (1, -1)),
            self.translated_subtree(tr, 1, 3, (1, 1)),
        ]
        .map(|tr| self.next_generation(depth - 1, tr));
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

    fn set_bit(&mut self, depth: usize, tr: TreeRef, p: (isize, isize)) -> TreeRef {
        let &tree = self.deref()(tr);
        match tree {
            Tree::Leaf { alive: false } => self.create_leaf(true),
            Tree::Leaf { alive: true } => tr,
            Tree::Branch { mut subtree } => {
                let (idx, p) = self.choose_subtree(depth, p);
                subtree[idx] = self.set_bit(depth - 1, subtree[idx], p);
                self.create_branch(subtree)
            }
        }
    }

    fn get_bit(&self, depth: usize, tr: TreeRef, p: (isize, isize)) -> bool {
        match *self.deref()(tr) {
            Tree::Leaf { alive } => alive,
            Tree::Branch { subtree } => {
                let (idx, p) = self.choose_subtree(depth, p);
                self.get_bit(depth - 1, subtree[idx], p)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::tests::GLIDER_STATES;
    use std::str::FromStr;

    const L3_CROSS: &'static str = "
        o      o
         o    o
          o  o  
           oo   
           oo   
          o  o  
         o    o 
        o      o";

    #[test]
    fn test_from() {
        let state = State::from_str(
            "
   oo       o
   o o       o
    o      ooo",
        )
        .unwrap();
        let hls: HashLifeState = state.clone().into();
        assert_eq!(Into::<State>::into(hls).normalize(), state);
    }

    #[test]
    fn test_from_depth_3() {
        let state = State::from_str(L3_CROSS).unwrap();
        let hls: HashLifeState = state.clone().into();
        assert_eq!(hls.depth, 3);
        assert_eq!(Into::<State>::into(hls).normalize(), state);
    }

    #[test]
    fn test_translated_subtree() {
        let HashLifeState {
            mut universe, root, ..
        } = State::from_str(L3_CROSS).unwrap().into();
        let hls = HashLifeState {
            root: universe.translated_subtree(root, 1, 3, (1, 1)),
            universe,
            depth: 2,
        };
        let state = State::from_str(
            "
           oo   
           oo
             o
              o",
        )
        .unwrap();
        assert_eq!(Into::<State>::into(hls), state);
    }

    #[test]
    fn test_single_step() {
        let [a, b] = [
            "ooo",
            "
            o
            o
            o",
        ];
        let state = State::from_str(a).unwrap();
        let mut hls: HashLifeState = state.into();
        assert_eq!(hls.depth, 2);
        hls.step();
        assert_eq!(hls.depth, 2);
        assert_eq!(
            Into::<State>::into(hls).normalize(),
            State::from_str(b).unwrap()
        );
    }

    #[test]
    fn test_glider() {
        // Test a boat + glider combo
        for (a, b) in GLIDER_STATES.into_iter().tuple_windows() {
            let mut a: HashLifeState = State::from_str(a).unwrap().into();
            a.step();
            let b = State::from_str(b).unwrap();
            assert_eq!(Into::<State>::into(a).normalize(), b);
        }
    }
}
