#[derive(Clone, Copy, PartialEq, Eq)]
pub struct P3 {
    pub y: isize,
    pub x: isize,
    pub z: usize,
}

impl P3 {
    pub fn new(y: isize, x: isize, z: usize) -> Self {
        Self { y, x, z }
    }

    pub fn origin(z: usize) -> Self {
        Self::new(0, 0, z)
    }

    pub fn within_tree(&self) -> bool {
        // y and x should be in the pyramid height z about (0, 0).
        let P3 { y, x, z } = *self;
        P3::origin(z).contains(P3 { y, x, z: 0 })
    }

    pub fn contains(&self, other: P3) -> bool {
        if other.z >= self.z {
            return self == &other;
        }
        let (rel_y, rel_x, rel_z) = (other.y - self.y, other.x - self.x, self.z - other.z);
        let w = 1 << (rel_z - 1);
        (-w..w).contains(&rel_y) && (-w..w).contains(&rel_x)
    }

    pub fn descend(&mut self) -> Option<usize> {
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

    pub fn quadrants(&self) -> Option<[Self; 4]> {
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

    #[test]
    fn test_doesnt_exist() {
        assert!(!P3::new(4, 4, 2).within_tree());
        assert!(!P3::new(4, 4, 3).within_tree());
        assert!(!P3::new(-1, -1, 0).within_tree());
    }

    #[test]
    fn test_exists() {
        assert!(P3::new(0, 0, 0).within_tree());
        assert!(P3::new(-4, -4, 3).within_tree());
    }

    #[test]
    fn test_contains() {
        assert!(P3::new(0, 0, 0).contains(P3::new(0, 0, 0)));
        assert!(P3::new(0, 0, 2).contains(P3::new(0, 0, 1)));
        assert!(P3::new(-2, -2, 2).contains(P3::new(-3, -3, 1)));
        assert!(!P3::new(-2, -2, 2).contains(P3::new(-1, -1, 1)));
    }
}
