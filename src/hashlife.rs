use std::{collections::HashMap, fmt::Display, str::FromStr, vec};

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
        while !P3::new(y, x, self.depth).floor_exists() {
            self.expand();
        }
        let p = P3::new(y, x, self.depth);
        self.root = self.universe.set_bit(self.root, p);
    }
}

impl IntoIterator for HashLife {
    type Item = (isize, isize);

    type IntoIter = vec::IntoIter<(isize, isize)>;

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

mod eq {
    use std::iter::zip;

    use super::*;

    struct Context<'hl> {
        tr: TreeRef,
        p: P3,
        hl: &'hl HashLife,
    }

    impl<'hl> Context<'hl> {
        fn new(hl: &'hl HashLife) -> Self {
            Context {
                tr: hl.root,
                p: P3::origin(hl.depth),
                hl,
            }
        }

        fn population(&self) -> usize {
            self.hl.universe.population(self.tr)
        }

        fn divide(&self) -> [Self; 4] {
            let ps = self.p.quadrants().unwrap();
            let subtree = self.hl.universe.subtree(self.tr);
            [0, 1, 2, 3]
                .map(|i| Self {
                    tr: subtree[i],
                    p: ps[i],
                    ..*self
                })
                .into()
        }
    }

    impl PartialEq for HashLife {
        fn eq(&self, other: &Self) -> bool {
            // Only time we'll check the depth
            if self.depth != other.depth {
                return false;
            }
            let mut a_b_cache: HashMap<TreeRef, TreeRef> = HashMap::new();
            let mut b_a_cache: HashMap<TreeRef, TreeRef> = HashMap::new();
            let mut stack = vec![(Context::new(self), Context::new(other))];
            while let Some((a, b)) = stack.pop() {
                // Early exit
                if a.population() != b.population() {
                    return false;
                }
                if a.population() == 0 || a.p.z == 0 {
                    continue;
                }
                // Check the cache
                let a_tr = b_a_cache.insert(b.tr, a.tr);
                let b_tr = a_b_cache.insert(a.tr, b.tr);
                match (a_tr, b_tr) {
                    (Some(a_tr), Some(b_tr)) => {
                        if a_tr != a.tr || b_tr != b.tr {
                            return false;
                        }
                    }
                    (None, None) => (),
                    _ => return false,
                }
                // Check the subtrees
                stack.extend(zip(a.divide(), b.divide()));
            }
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_state::tests::GLIDER_STATES;
    use itertools::Itertools;
    use std::str::FromStr;

    fn dedent(s: &str) -> String {
        let get_indent = |s: &str| match s.trim_start().len() {
            0 => None,
            l => Some(s.len() - l),
        };
        let s = s.trim_end();
        let indent = s.lines().filter_map(get_indent).min().unwrap_or_default();
        let lines = s.lines().skip_while(|l| l.trim().is_empty());
        lines.map(|l| l.split_at(indent).1.trim_end()).join("\n")
    }

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
        let a = HashLife::from_str("o").unwrap();
        let b = HashLife::from_str("o").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_from_depth_3() {
        let hl = HashLife::from_str(L3_CROSS).unwrap();
        assert_eq!(hl.depth, 3);
        assert_eq!(hl.to_string(), dedent(L3_CROSS));
    }

    #[test]
    fn test_reframe() {
        let HashLife {
            mut universe, root, ..
        } = HashLife::from_str(L3_CROSS).unwrap();
        let hl = HashLife {
            root: universe.reframe(root, P3 { y: 1, x: 1, z: 3 }, 2),
            universe,
            depth: 2,
        };
        let expected = dedent(
            "
           oo   
           oo
             o
              o",
        );
        assert_eq!(hl.to_string(), expected);
    }

    #[test]
    fn test_single_step() {
        let mut hl = HashLife::from_str("ooo").unwrap();
        assert_eq!(hl.depth, 2);
        hl.step(0);
        assert_eq!(hl.to_string(), "ooo".chars().join("\n"));
    }

    #[test]
    fn test_glider() {
        // Test a boat + glider combo
        for (a, b) in GLIDER_STATES.into_iter().tuple_windows() {
            let mut a = HashLife::from_str(a).unwrap();
            a.step(0);
            assert_eq!(a.to_string(), dedent(b));
        }
    }

    #[test]
    fn test_glider_superspeed() {
        // Test that advancing once in a big step is the same as doing a small
        // step several times.
        let mut a = HashLife::from_str(GLIDER_STATES[0]).unwrap();
        let mut b = a.clone();
        let log2_steps = 6;
        a.step(log2_steps);
        for _ in 0..1 << log2_steps {
            b.step(0);
        }
        assert_eq!(a, b);
    }

    #[test]
    fn test_glider_pop() {
        // Test population is maintained over many steps.
        let mut a = HashLife::from_str(GLIDER_STATES[0]).unwrap();
        let pop1 = a.universe.population(a.root);
        a.step(50);
        a.step(50);
        let pop2 = a.universe.population(a.root);
        assert_eq!(pop1, pop2);
    }
}
