//! Sport-agnostic promotion and relegation across a pyramid of divisions.
//!
//! The mechanism is pure ranking logic — the bottom `n` teams of a division swap with the
//! top `n` of the division below — so it has no sport-specific content and lives here. The
//! sport only supplies each division's *final standings order* (which it computes with its
//! own table rules); the pyramid does the rest. Any league sport reuses it unchanged.

/// A league pyramid: division membership, top flight first (`divisions[0]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pyramid {
    pub divisions: Vec<Vec<u32>>,
}

impl Pyramid {
    pub fn new(divisions: Vec<Vec<u32>>) -> Self {
        Self { divisions }
    }

    pub fn len(&self) -> usize {
        self.divisions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.divisions.is_empty()
    }

    /// The division index a team currently sits in, if any.
    pub fn division_of(&self, team: u32) -> Option<usize> {
        self.divisions.iter().position(|d| d.contains(&team))
    }

    /// Apply one season's promotion and relegation, producing next season's membership.
    ///
    /// `orders[d]` is division `d`'s final standings, best-first, and must be a permutation of
    /// that division's current members. The bottom `n` of each division are relegated and swap
    /// with the top `n` of the division below. Division sizes are preserved.
    pub fn promote_relegate(&mut self, orders: &[Vec<u32>], n: usize) {
        assert_eq!(orders.len(), self.divisions.len(), "one ranking per division required");
        for (d, order) in orders.iter().enumerate() {
            assert!(2 * n <= order.len(), "swap count {n} too large for division {d}");
        }

        let num = orders.len();
        let mut next = vec![Vec::new(); num];
        for d in 0..num {
            let order = &orders[d];
            let len = order.len();
            // Stayers: drop the top `n` (promoted out, unless this is the top division) and the
            // bottom `n` (relegated out, unless this is the bottom division).
            let start = if d > 0 { n } else { 0 };
            let end = if d + 1 < num { len - n } else { len };
            next[d].extend_from_slice(&order[start..end]);
            // Promoted up from the division below: its top `n`.
            if d + 1 < num {
                next[d].extend_from_slice(&orders[d + 1][..n]);
            }
            // Relegated down from the division above: its bottom `n`.
            if d > 0 {
                let above = &orders[d - 1];
                next[d].extend_from_slice(&above[above.len() - n..]);
            }
        }
        self.divisions = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_bottom_with_top_of_the_division_below() {
        let mut p = Pyramid::new(vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7]]);
        // Final orders (best-first): same as listed.
        p.promote_relegate(&[vec![0, 1, 2, 3], vec![4, 5, 6, 7]], 1);
        // Top: keep 0,1,2 (3 relegated) + 4 promoted.
        assert_eq!(p.divisions[0], vec![0, 1, 2, 4]);
        // Bottom: keep 5,6,7 (4 promoted) + 3 relegated.
        assert_eq!(p.divisions[1], vec![5, 6, 7, 3]);
    }

    #[test]
    fn middle_division_exchanges_both_ways() {
        let mut p = Pyramid::new(vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7], vec![8, 9, 10, 11]]);
        p.promote_relegate(&[vec![0, 1, 2, 3], vec![4, 5, 6, 7], vec![8, 9, 10, 11]], 1);
        // Middle: drop top 4 (promoted up) and bottom 7 (relegated down); keep 5,6.
        // Gains 8 (top of div below) and 3 (bottom of div above).
        assert_eq!(p.divisions[1], vec![5, 6, 8, 3]);
        assert_eq!(p.divisions[0], vec![0, 1, 2, 4]); // 3 down, 4 up
        assert_eq!(p.divisions[2], vec![9, 10, 11, 7]); // 8 up, 7 down
    }

    #[test]
    fn division_sizes_are_preserved() {
        let mut p = Pyramid::new(vec![(0..6).collect(), (6..12).collect(), (12..18).collect()]);
        let orders: Vec<Vec<u32>> = p.divisions.clone();
        p.promote_relegate(&orders, 2);
        assert!(p.divisions.iter().all(|d| d.len() == 6));
        // Every team still appears exactly once across the pyramid.
        let mut all: Vec<u32> = p.divisions.iter().flatten().copied().collect();
        all.sort_unstable();
        assert_eq!(all, (0..18).collect::<Vec<_>>());
    }

    #[test]
    fn division_of_finds_the_team() {
        let p = Pyramid::new(vec![vec![0, 1], vec![2, 3]]);
        assert_eq!(p.division_of(0), Some(0));
        assert_eq!(p.division_of(3), Some(1));
        assert_eq!(p.division_of(9), None);
    }
}
