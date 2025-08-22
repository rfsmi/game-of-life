use std::{collections::HashMap, iter::zip};

use crate::{HashLife, p3::P3, universe::TreeRef};

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
