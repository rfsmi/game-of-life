mod basic_state;
mod eq;
mod p3;
pub mod render;
mod universe;

#[cfg(test)]
mod tests;

use std::{fmt::Display, str::FromStr};

use crate::{
    basic_state::BasicState,
    p3::P3,
    universe::{TreeRef, Universe},
};

#[derive(Clone, Debug)]
pub struct HashLife {
    universe: Universe,
    depth: usize,
    root: TreeRef,
}

impl HashLife {
    pub fn new() -> Self {
        let mut universe = Universe::default();
        let depth = 2;
        let root = universe.empty_tree(depth);
        Self {
            universe,
            depth,
            root,
        }
    }

    pub fn step(&mut self, log2_steps: usize) {
        let superspeed_depth = log2_steps + 2;
        while self.depth < superspeed_depth - 1 {
            self.expand();
        }
        // We can only step if all the border nodes in the 4x4 square are empty.
        let center = self.universe.reframe(self.root, P3::origin(2), 1);
        if self.universe.population(center) != self.universe.population(self.root) {
            self.expand();
        }
        self.expand();
        self.root = self.universe.step(self.root, self.depth, superspeed_depth);
        self.depth -= 1;
    }
}

impl HashLife {
    fn expand(&mut self) {
        self.root = self.universe.expand_universe(self.depth, self.root);
        self.depth += 1;
    }

    fn set_bit(&mut self, (y, x): (isize, isize)) {
        while !P3::new(y, x, self.depth).within_tree() {
            self.expand();
        }
        let p = P3::new(y, x, self.depth);
        self.root = self.universe.set_bit(self.root, p);
    }
}

impl IntoIterator for HashLife {
    type Item = (isize, isize);

    type IntoIter = std::vec::IntoIter<(isize, isize)>;

    fn into_iter(self) -> Self::IntoIter {
        let mut cells = Vec::with_capacity(self.universe.population(self.root));
        let mut stack = vec![(self.root, P3::origin(self.depth))];
        while let Some((tr, p)) = stack.pop() {
            if self.universe.population(tr) == 0 {
                continue;
            }
            if let Some(ps) = p.quadrants() {
                stack.extend(self.universe.subtree(tr).into_iter().zip(ps));
            } else if self.universe.alive(tr) {
                cells.push((p.y, p.x));
            }
        }
        cells.into_iter()
    }
}

impl Display for HashLife {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cells = self.clone().into_iter().collect();
        BasicState { cells }.fmt(f)
    }
}

impl FromIterator<(isize, isize)> for HashLife {
    fn from_iter<T: IntoIterator<Item = (isize, isize)>>(iter: T) -> Self {
        let mut hl = HashLife::new();
        for p in iter {
            hl.set_bit(p);
        }
        hl
    }
}

impl FromStr for HashLife {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BasicState::from_str(s).map(|s| s.into_iter().collect())
    }
}
