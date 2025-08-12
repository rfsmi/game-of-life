use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write},
    ops::Range,
    str::FromStr,
};

use itertools::Itertools;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct State {
    cells: HashSet<(isize, isize)>,
}

impl State {
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
            if let (2..=3, true) | (3, false) = (count, alive) {
                cells.insert(p);
            }
        }
        Self { cells }
    }

    pub fn normalize(mut self) -> Self {
        let Span::Covers { ys, xs } = self.span() else {
            return self;
        };
        if (ys.start, xs.start) != (0, 0) {
            for (y, x) in std::mem::take(&mut self.cells) {
                self.set_bit((y - ys.start, x - xs.start));
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

impl State {
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

impl FromStr for State {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut state = State::default();
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

impl Display for State {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boat() {
        // Boat is constant.
        let state = State::from_str(
            "
            oo
            o o
             o
        ",
        )
        .unwrap();
        assert_eq!(state.step().normalize(), state);
    }

    #[test]
    fn test_blinker() {
        // Blinker blinks with period 2.
        let state_1 = State::from_str("ooo").unwrap();
        let state_2 = State::from_str(
            "
            o
            o
            o
        ",
        )
        .unwrap();
        assert_ne!(state_1, state_2);
        assert_eq!(state_1.step().normalize(), state_2);
        assert_eq!(state_2.step().normalize(), state_1);
    }

    #[test]
    fn test_glider() {
        // Test that the glider moves down and right.
        let mut state = State::from_str(
            "
             o
              o
            ooo
        ",
        )
        .unwrap();
        for i in 0..100 {
            state = state.step();
            let Span::Covers { ys, xs } = state.span() else {
                panic!();
            };
            let (dy, dx) = (1, if i % 4 >= 2 { 1 } else { 0 });
            assert_eq!((ys.start, xs.start), (i / 4 + dy, i / 4 + dx));
        }
    }
}
