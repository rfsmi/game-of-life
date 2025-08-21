use std::collections::HashMap;

use crate::p3::P3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TreeRef(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Tree {
    Leaf(bool),
    Branch([TreeRef; 4]),
}

#[derive(Default, Clone, Debug)]
pub struct Universe {
    nodes: Vec<Tree>,
    empty_trees: Vec<TreeRef>,
    populations: Vec<usize>,
    next_gen: HashMap<(TreeRef, bool), TreeRef>,
    interned_nodes: HashMap<Tree, TreeRef>,
}

impl Universe {
    pub fn empty_tree(&mut self, depth: usize) -> TreeRef {
        while self.empty_trees.len() <= depth {
            let tr = match self.empty_trees.last() {
                Some(&tr) => self.canonicalise(Tree::Branch([tr, tr, tr, tr])),
                None => self.canonicalise(Tree::Leaf(false)),
            };
            self.empty_trees.push(tr);
        }
        self.empty_trees[depth]
    }

    pub fn get_node(&self, mut tr: TreeRef, mut p: P3) -> TreeRef {
        while let Some(i) = p.descend() {
            tr = self.subtree(tr)[i];
        }
        tr
    }

    pub fn set_bit(&mut self, mut tr: TreeRef, mut p: P3) -> TreeRef {
        let mut stack = vec![];
        while let Some(i) = p.descend() {
            let subtree = self.subtree(tr);
            stack.push((subtree, i));
            tr = subtree[i];
        }
        stack.into_iter().rev().fold(
            self.canonicalise(Tree::Leaf(true)),
            |tr, (mut subtree, i)| {
                subtree[i] = tr;
                self.canonicalise(Tree::Branch(subtree))
            },
        )
    }

    pub fn expand_universe(&mut self, level: usize, tr: TreeRef) -> TreeRef {
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

    pub fn alive(&self, TreeRef(i): TreeRef) -> bool {
        match self.nodes[i] {
            Tree::Leaf(alive) => alive,
            Tree::Branch(..) => panic!(),
        }
    }

    pub fn subtree(&self, TreeRef(i): TreeRef) -> [TreeRef; 4] {
        match self.nodes[i] {
            Tree::Leaf(..) => panic!(),
            Tree::Branch(subtree) => subtree,
        }
    }

    pub fn population(&self, TreeRef(i): TreeRef) -> usize {
        self.populations[i]
    }

    pub fn reframe(&mut self, tr: TreeRef, p: P3, z: usize) -> TreeRef {
        // Get the tree with the node at p (w.r.t. tr) centered at depth z.
        let (z, p) = (p.z, P3 { z, ..p });
        enum State {
            Reframe(P3),
            Canonicalise,
        }
        let mut done = vec![];
        let mut todo = vec![State::Reframe(p)];
        while let Some(state) = todo.pop() {
            match state {
                State::Reframe(p) => match p.quadrants() {
                    Some(ps) => {
                        todo.push(State::Canonicalise);
                        todo.extend(ps.map(State::Reframe));
                    }
                    None => done.push(self.get_node(tr, P3 { z, ..p })),
                },
                State::Canonicalise => {
                    let subtree = [done.pop(), done.pop(), done.pop(), done.pop()];
                    let tr = self.canonicalise(Tree::Branch(subtree.map(Option::unwrap)));
                    done.push(tr);
                }
            }
        }
        done.pop().unwrap()
    }

    pub fn step(&mut self, tr: TreeRef, depth: usize, superspeed_depth: usize) -> TreeRef {
        enum State {
            Step(TreeRef, usize),
            Push9(TreeRef, usize),
            Pop9Into4(usize),
            Pop4Into1,
            UpdateCache((TreeRef, bool)),
        }
        let mut done = vec![];
        let mut stack = vec![State::Step(tr, depth)];
        while let Some(state) = stack.pop() {
            match state {
                State::Step(tr, depth) => {
                    let key = (tr, depth <= superspeed_depth);
                    if let Some(&tr) = self.next_gen.get(&key) {
                        done.push(tr);
                    } else {
                        stack.push(State::UpdateCache(key));
                        stack.push(State::Push9(tr, depth));
                    }
                }
                State::Push9(tr, 2) => {
                    let bitmask = self.make_l2_bitmask(tr);
                    done.push(self.l2_gen(bitmask));
                }
                State::Push9(tr, depth) => {
                    let l2_trees = [0, 1, 2, 3, 4, 5, 6, 7, 8]
                        .map(|i| (i / 3 * 2 - 2, i % 3 * 2 - 2))
                        .map(|(y, x)| self.reframe(tr, P3 { y, x, z: 3 }, 2));
                    stack.push(State::Pop4Into1);
                    stack.push(State::Pop9Into4(depth));
                    if depth <= superspeed_depth {
                        let subtree = l2_trees.map(|l2| State::Step(l2, depth - 1));
                        stack.extend(subtree);
                    } else {
                        let subtree = l2_trees.map(|l2| self.reframe(l2, P3::origin(2), 1));
                        done.extend(subtree.into_iter().rev());
                    }
                }
                State::Pop9Into4(depth) => {
                    let l1_trees = [
                        [done.pop(), done.pop(), done.pop()].map(Option::unwrap),
                        [done.pop(), done.pop(), done.pop()].map(Option::unwrap),
                        [done.pop(), done.pop(), done.pop()].map(Option::unwrap),
                    ];
                    let l2_trees = [0, 1, 3, 4]
                        .map(|i| [0, 1, 3, 4].map(|j| l1_trees[(i + j) / 3][(i + j) % 3]))
                        .map(|subtree| self.canonicalise(Tree::Branch(subtree)));
                    let subtree = l2_trees.map(|l2| State::Step(l2, depth - 1));
                    stack.extend(subtree);
                }
                State::Pop4Into1 => {
                    let subtree = [done.pop(), done.pop(), done.pop(), done.pop()];
                    let tr = self.canonicalise(Tree::Branch(subtree.map(Option::unwrap)));
                    done.push(tr);
                }
                State::UpdateCache(key) => {
                    self.next_gen.insert(key, *done.last().unwrap());
                }
            }
        }
        done.pop().unwrap()
    }
}

impl Universe {
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
}
