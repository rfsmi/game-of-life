use std::{
    collections::{HashMap, HashSet, hash_set},
    fmt::{Display, Write},
    ops::Range,
    str::FromStr,
};

use itertools::Itertools;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct BasicState {
    pub cells: HashSet<(isize, isize)>,
}

impl BasicState {
    pub fn set_bit(&mut self, p: (isize, isize)) {
        self.cells.insert(p);
    }

    pub fn step(&self) -> Self {
        let mut counts = HashMap::new();
        for &p1 in &self.cells {
            for p2 in neighbours(p1) {
                *counts.entry(p2).or_default() += 1;
            }
        }
        let mut cells = HashSet::new();
        for (p, count) in counts {
            let alive = self.cells.contains(&p);
            if let (2 | 3, true) | (3, false) = (count, alive) {
                cells.insert(p);
            }
        }
        Self { cells }
    }

    pub fn normalize(mut self) -> Self {
        let Span::Covers { ys, xs } = self.span() else {
            return self;
        };
        let dy = ys.start + ys.len() as isize / 2;
        let dx = xs.start + xs.len() as isize / 2;
        if (dy, dx) != (0, 0) {
            for (y, x) in std::mem::take(&mut self.cells) {
                self.set_bit((y - dy, x - dx));
            }
        }
        self
    }
}

fn neighbours((y, x): (isize, isize)) -> impl Iterator<Item = (isize, isize)> {
    (-1..=1)
        .cartesian_product(-1..=1)
        .filter(|&d| d != (0, 0))
        .map(move |(dy, dx)| (y + dy, x + dx))
}

enum Span {
    Empty,
    Covers { ys: Range<isize>, xs: Range<isize> },
}

impl BasicState {
    fn span(&self) -> Span {
        use Span::*;
        let xs = match self.cells.iter().map(|(_, x)| *x).minmax() {
            itertools::MinMaxResult::NoElements => return Empty,
            itertools::MinMaxResult::OneElement(x) => x..x + 1,
            itertools::MinMaxResult::MinMax(x1, x2) => x1..x2 + 1,
        };
        let ys = match self.cells.iter().map(|(y, _)| *y).minmax() {
            itertools::MinMaxResult::NoElements => return Empty,
            itertools::MinMaxResult::OneElement(y) => y..y + 1,
            itertools::MinMaxResult::MinMax(y1, y2) => y1..y2 + 1,
        };
        Covers { ys, xs }
    }
}

impl IntoIterator for BasicState {
    type Item = (isize, isize);

    type IntoIter = hash_set::IntoIter<(isize, isize)>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

impl FromIterator<(isize, isize)> for BasicState {
    fn from_iter<T: IntoIterator<Item = (isize, isize)>>(iter: T) -> Self {
        Self {
            cells: iter.into_iter().collect(),
        }
    }
}

impl FromStr for BasicState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut state = BasicState::default();
        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                match c {
                    ' ' => (),
                    'o' => {
                        state.set_bit((y as isize, x as isize));
                    }
                    _ => return Err(format!("Unexpected character {c}")),
                }
            }
        }
        Ok(state.normalize())
    }
}

impl Display for BasicState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Span::Covers { ys, xs } = self.span() else {
            return Ok(());
        };
        let (mut current_y, mut current_x) = (ys.start, xs.start);
        for &(y, x) in self.cells.iter().sorted() {
            while current_y < y {
                f.write_char('\n')?;
                current_x = xs.start;
                current_y += 1;
            }
            f.write_str(&" ".repeat(x.abs_diff(current_x)))?;
            f.write_char('o')?;
            current_x = x + 1;
        }
        Ok(())
    }
}
