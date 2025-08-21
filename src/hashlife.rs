use std::collections::HashMap;

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

    fn expand(&mut self) {
        self.root = self.universe.expand_universe(self.depth, self.root);
        self.depth += 1;
    }

    fn set_bit(&mut self, (y, x): (isize, isize)) {
        while !self.contains((y, x)) {
            self.expand();
        }
        let z = self.depth;
        self.root = self.universe.set_bit(self.root, P3 { y, x, z });
    }

    fn get_bit(&self, (y, x): (isize, isize)) -> bool {
        let z = self.depth;
        let tr = self.universe.get_node(self.root, P3 { y, x, z });
        self.universe.alive(tr)
    }

    fn step(&mut self, log2_steps: usize) {
        let superspeed_depth = log2_steps + 2;
        while self.depth < superspeed_depth {
            self.expand();
        }
        // We can only step if all the border nodes in the 4x4 square are empty.
        let center = self.universe.reframe(self.root, P3 { y: 0, x: 0, z: 2 }, 1);
        if self.universe.population(center) != self.universe.population(self.root) {
            self.expand();
        }
        self.expand();
        self.root = self.universe.step(self.root, self.depth, superspeed_depth);
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
        let p = P3 {
            y: 0,
            x: 0,
            z: hls.depth,
        };
        let mut stack = vec![(hls.root, p)];
        while let Some((tr, p)) = stack.pop() {
            if hls.universe.population(tr) == 0 {
                continue;
            }
            if let Some(ps) = p.quadrants() {
                stack.extend(hls.universe.subtree(tr).into_iter().zip(ps));
            } else if hls.universe.alive(tr) {
                state.cells.insert((p.y, p.x));
            }
        }
        state
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Tree {
    Leaf(bool),
    Branch([TreeRef; 4]),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TreeRef(usize);

#[derive(Default, Clone)]
struct Universe {
    nodes: Vec<Tree>,
    populations: Vec<usize>,
    next_gen: HashMap<(TreeRef, bool), TreeRef>,
    interned_nodes: HashMap<Tree, TreeRef>,
}

impl Universe {
    fn empty_tree(&mut self, depth: usize) -> TreeRef {
        if depth == 0 {
            return self.canonicalise(Tree::Leaf(false));
        }
        let tr = self.empty_tree(depth - 1);
        self.canonicalise(Tree::Branch([tr, tr, tr, tr]))
    }

    fn get_node(&self, mut tr: TreeRef, mut p: P3) -> TreeRef {
        while let Some(i) = p.descend() {
            tr = self.subtree(tr)[i];
        }
        tr
    }

    fn set_bit(&mut self, tr: TreeRef, mut p: P3) -> TreeRef {
        let Some(i) = p.descend() else {
            return self.canonicalise(Tree::Leaf(true));
        };
        let mut subtree = self.subtree(tr);
        subtree[i] = self.set_bit(subtree[i], p);
        self.canonicalise(Tree::Branch(subtree))
    }

    fn expand_universe(&mut self, level: usize, tr: TreeRef) -> TreeRef {
        let [nw, ne, sw, se] = self.subtree(tr);
        let border = self.empty_tree(level - 1);
        let subtree = [
            self.canonicalise(Tree::Branch([border, border, border, nw])),
            self.canonicalise(Tree::Branch([border, border, ne, border])),
            self.canonicalise(Tree::Branch([border, sw, border, border])),
            self.canonicalise(Tree::Branch([se, border, border, border])),
        ];
        self.canonicalise(Tree::Branch(subtree))
    }
}

impl Universe {
    fn alive(&self, TreeRef(i): TreeRef) -> bool {
        match self.nodes[i] {
            Tree::Leaf(alive) => alive,
            Tree::Branch(..) => panic!(),
        }
    }

    fn subtree(&self, TreeRef(i): TreeRef) -> [TreeRef; 4] {
        match self.nodes[i] {
            Tree::Leaf(..) => panic!(),
            Tree::Branch(subtree) => subtree,
        }
    }

    fn population(&self, TreeRef(i): TreeRef) -> usize {
        self.populations[i]
    }

    fn canonicalise(&mut self, tree: Tree) -> TreeRef {
        *self.interned_nodes.entry(tree).or_insert_with_key(|&tree| {
            let population = match tree {
                Tree::Leaf(true) => 1,
                Tree::Leaf(false) => 0,
                Tree::Branch(subtree) => subtree.map(|TreeRef(i)| self.populations[i]).iter().sum(),
            };
            self.populations.push(population);
            self.nodes.push(tree);
            TreeRef(self.nodes.len() - 1)
        })
    }
}

impl Universe {
    fn l2_gen(&mut self, bitmask: u16) -> TreeRef {
        fn leaf(bitmask: u16) -> Tree {
            let center = bitmask & 0b0000_0010_0000 != 0;
            let neighbours = (bitmask & 0b0111_0101_0111).count_ones();
            match (center, neighbours) {
                (true, 2 | 3) | (false, 3) => Tree::Leaf(true),
                _ => Tree::Leaf(false),
            }
        }
        let subtree = [
            self.canonicalise(leaf(bitmask >> 5)),
            self.canonicalise(leaf(bitmask >> 4)),
            self.canonicalise(leaf(bitmask >> 1)),
            self.canonicalise(leaf(bitmask >> 0)),
        ];
        self.canonicalise(Tree::Branch(subtree))
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

    fn reframe(&mut self, tr: TreeRef, p: P3, z: usize) -> TreeRef {
        // Get the tree with the node at p (w.r.t. tr) centered at depth z.
        if let Some(ps) = (P3 { z, ..p }).quadrants() {
            let subtree = ps.map(|P3 { y, x, z }| self.reframe(tr, P3 { y, x, ..p }, z));
            self.canonicalise(Tree::Branch(subtree))
        } else {
            return self.get_node(tr, p);
        }
    }

    fn step(&mut self, tr: TreeRef, depth: usize, superspeed_depth: usize) -> TreeRef {
        let key = (tr, depth <= superspeed_depth);
        if let Some(&tr) = self.next_gen.get(&key) {
            return tr;
        }
        if depth == 2 {
            let bitmask = self.make_l2_bitmask(tr);
            return self.l2_gen(bitmask);
        }
        let l1_trees = [0, 1, 2, 3, 4, 5, 6, 7, 8]
            .map(|i| (i / 3 * 2 - 2, i % 3 * 2 - 2))
            .map(|(y, x)| self.reframe(tr, P3 { y, x, z: 3 }, 2))
            .map(|l2| self.step(l2, depth - 1, superspeed_depth));
        let l2_trees = [0, 1, 3, 4]
            .map(|i| [0, 1, 3, 4].map(|j| l1_trees[i + j]))
            .map(|subtree| self.canonicalise(Tree::Branch(subtree)));
        let subtree = if depth <= superspeed_depth {
            l2_trees.map(|l2| self.step(l2, depth - 1, superspeed_depth))
        } else {
            l2_trees.map(|l2| self.reframe(l2, P3 { y: 0, x: 0, z: 2 }, 1))
        };
        let tr = self.canonicalise(Tree::Branch(subtree));
        self.next_gen.insert(key, tr);
        tr
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

    fn quadrants(&self) -> Option<[Self; 4]> {
        if self.z == 0 {
            None
        } else {
            let (pos, neg) = (1 << self.z >> 2, -1 << self.z >> 2);
            let quantrants = [
                (self.y + neg, self.x + neg, self.z - 1),
                (self.y + neg, self.x + pos, self.z - 1),
                (self.y + pos, self.x + neg, self.z - 1),
                (self.y + pos, self.x + pos, self.z - 1),
            ];
            Some(quantrants.map(|(y, x, z)| Self { y, x, z }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::tests::GLIDER_STATES;
    use itertools::Itertools;
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
        assert_eq!(State::normalize(hls.into()), state);
    }

    #[test]
    fn test_from_depth_3() {
        let state = State::from_str(L3_CROSS).unwrap();
        let hls: HashLifeState = state.clone().into();
        assert_eq!(hls.depth, 3);
        assert_eq!(State::normalize(hls.into()), state);
    }

    #[test]
    fn test_reframe() {
        let HashLifeState {
            mut universe, root, ..
        } = State::from_str(L3_CROSS).unwrap().into();
        let hls = HashLifeState {
            root: universe.reframe(root, P3 { y: 1, x: 1, z: 3 }, 2),
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
        hls.step(0);
        assert_eq!(State::normalize(hls.into()), State::from_str(b).unwrap());
    }

    #[test]
    fn test_glider() {
        // Test a boat + glider combo
        for (a, b) in GLIDER_STATES.into_iter().tuple_windows() {
            let mut a: HashLifeState = State::from_str(a).unwrap().into();
            a.step(0);
            let b = State::from_str(b).unwrap();
            assert_eq!(State::normalize(a.into()), b);
        }
    }

    #[test]
    fn test_glider_superspeed() {
        // Test that advancing once in a big step is the same as doing a small
        // step several times.
        let mut a: HashLifeState = State::from_str(GLIDER_STATES[0]).unwrap().into();
        let mut b = a.clone();
        let log2_steps = 6;
        a.step(log2_steps);
        let a = State::normalize(a.into());
        for _ in 0..1 << log2_steps {
            b.step(0);
        }
        assert_eq!(a, State::normalize(b.into()));
    }
}
