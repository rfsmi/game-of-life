use crate::{HashLife, basic_state::BasicState, p3::P3};
use itertools::Itertools;
use std::str::FromStr;

const GLIDER: [&'static str; 6] = [
    "
   oo       o
   o o       o
    o      ooo",
    "
   oo
   o o     o o
    o       oo
            o",
    "
   oo 
   o o       o
    o      o o
            oo",
    "
   oo
   o o      o
    o        oo
            oo",
    "
   oo
   o o       o
    o         o
            ooo",
    "
   oo
   o o
    o       o o
             oo
             o",
];

const L3_CROSS: &'static str = "
        o      o
         o    o
          o  o  
           oo   
           oo   
          o  o  
         o    o 
        o      o";

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

mod basic_state {
    use super::*;

    #[test]
    fn test_boat() {
        // Boat is constant.
        let state = BasicState::from_str(
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
        let state_1 = BasicState::from_str("ooo").unwrap();
        let state_2 = BasicState::from_str(
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
        // Test a boat + glider combo
        for (a, b) in GLIDER.into_iter().tuple_windows() {
            let a = BasicState::from_str(a).unwrap().step().normalize();
            let b = BasicState::from_str(b).unwrap();
            assert_eq!(a, b);
        }
    }
}

mod hash_life {
    use super::*;

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
        for (a, b) in GLIDER.into_iter().tuple_windows() {
            let mut a = HashLife::from_str(a).unwrap();
            a.step(0);
            assert_eq!(a.to_string(), dedent(b));
        }
    }

    #[test]
    fn test_glider_superspeed() {
        // Test that advancing once in a big step is the same as doing a small
        // step several times.
        let mut a = HashLife::from_str(GLIDER[0]).unwrap();
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
        let mut a = HashLife::from_str(GLIDER[0]).unwrap();
        let pop1 = a.universe.population(a.root);
        a.step(50);
        a.step(50);
        let pop2 = a.universe.population(a.root);
        assert_eq!(pop1, pop2);
    }
}
