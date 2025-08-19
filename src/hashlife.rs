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
        let z = self.depth;
        self.root = self.universe.set_bit(self.root, P3 { y, x, z });
    }

    fn get_bit(&self, (y, x): (isize, isize)) -> bool {
        let z = self.depth;
        let tr = self.universe.get_node(self.root, P3 { y, x, z });
        self.universe.alive(tr)
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
        .map(|(y, x)| P3 { y, x, z: 2 })
        .any(|p| self.universe.get_node(self.root, p) != empty)
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
        let w = 1 << (hls.depth - 1);
        (-w..w)
            .cartesian_product(-w..w)
            .filter(|&p| hls.get_bit(p))
            .for_each(|p| state.set_bit(p));
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
    fn empty_tree(&mut self, depth: usize) -> TreeRef {
        if depth == 0 {
            return self.create_leaf(false);
        }
        let tr = self.empty_tree(depth - 1);
        self.create_branch([tr, tr, tr, tr])
    }

    fn get_node(&self, mut tr: TreeRef, mut p: P3) -> TreeRef {
        while let Some(i) = p.descend() {
            tr = self.subtree(tr)[i];
        }
        tr
    }

    fn set_bit(&mut self, tr: TreeRef, mut p: P3) -> TreeRef {
        let Some(i) = p.descend() else {
            return self.create_leaf(true);
        };
        let mut subtree = self.subtree(tr);
        subtree[i] = self.set_bit(subtree[i], p);
        self.create_branch(subtree)
    }

    fn expand_universe(&mut self, level: usize, tr: TreeRef) -> TreeRef {
        let [nw, ne, sw, se] = self.subtree(tr);
        let border = self.empty_tree(level - 1);
        let subtree = [
            self.create_branch([border, border, border, nw]),
            self.create_branch([border, border, ne, border]),
            self.create_branch([border, sw, border, border]),
            self.create_branch([se, border, border, border]),
        ];
        self.create_branch(subtree)
    }
}

impl Universe {
    fn alive(&self, TreeRef(i): TreeRef) -> bool {
        match self.nodes[i] {
            Tree::Leaf { alive } => alive,
            Tree::Branch { .. } => panic!(),
        }
    }

    fn subtree(&self, TreeRef(i): TreeRef) -> [TreeRef; 4] {
        match self.nodes[i] {
            Tree::Leaf { .. } => panic!(),
            Tree::Branch { subtree } => subtree,
        }
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
                let tr = self.get_node(tr, P3 { y, x, z: 2 });
                if self.alive(tr) {
                    bitmask += 1;
                }
            }
        }
        bitmask
    }

    fn tree_at(&mut self, p: P3, wrt: (TreeRef, usize)) -> TreeRef {
        let Some(ps) = p.quarter() else {
            let (tr, z) = wrt;
            return self.get_node(tr, P3 { z, ..p });
        };
        let subtree = ps.map(|p| self.tree_at(p, wrt));
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
            self.tree_at(P3 { y: -1, x: -1, z: 2 }, (tr, 3)),
            self.tree_at(P3 { y: -1, x: 1, z: 2 }, (tr, 3)),
            self.tree_at(P3 { y: 1, x: -1, z: 2 }, (tr, 3)),
            self.tree_at(P3 { y: 1, x: 1, z: 2 }, (tr, 3)),
        ]
        .map(|tr| self.next_generation(depth - 1, tr));
        let next_tr = self.create_branch(subtree);
        self.next_gen.insert(tr, next_tr);
        next_tr
    }
}

#[derive(Clone, Copy)]
struct P3 {
    y: isize,
    x: isize,
    z: usize,
}

impl P3 {
    fn descend(&mut self) -> Option<usize> {
        if self.z == 0 {
            return None;
        }
        let w = (1 << self.z) / 4;
        let (i, dy, dx) = match (self.y, self.x) {
            (..0, ..0) => (0, w, w),
            (..0, 0..) => (1, w, -w),
            (0.., ..0) => (2, -w, w),
            (0.., 0..) => (3, -w, -w),
        };
        if self.z == 1 {
            (self.y, self.x) = (0, 0);
        } else {
            (self.y, self.x) = (self.y + dy, self.x + dx);
        }
        self.z -= 1;
        Some(i)
    }

    fn quarter(self) -> Option<[Self; 4]> {
        let Self { y, x, z } = self;
        let yxs = match z {
            0 => return None,
            1 => [(y - 1, x - 1), (y - 1, x), (y, x - 1), (y, x)],
            _ => {
                let w = 1 << (z - 2);
                [
                    (y - w, x - w),
                    (y - w, x + w),
                    (y + w, x - w),
                    (y + w, x + w),
                ]
            }
        };
        Some(yxs.map(|(y, x)| Self { y, x, z: z - 1 }))
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
    fn test_single() {
        let state = State::from_str("o").unwrap();
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
    fn test_tree_at() {
        let HashLifeState {
            mut universe, root, ..
        } = State::from_str(L3_CROSS).unwrap().into();
        let hls = HashLifeState {
            root: universe.tree_at(P3 { y: 1, x: 1, z: 2 }, (root, 3)),
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
