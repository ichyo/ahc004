mod text_scanner;

const LEN: u8 = 20u8;

use rand::distributions::{Distribution, Standard};
use rand::prelude::*;
use std::ops::Index;
use std::ops::IndexMut;
use std::time::Duration;
use std::time::Instant;
use text_scanner::scan;

struct Matrix<T>(Vec<Vec<T>>);

impl<T> Index<Pos> for Matrix<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.0[index.r as usize][index.c as usize]
    }
}

impl<T> IndexMut<Pos> for Matrix<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.0[index.r as usize][index.c as usize]
    }
}

#[derive(Debug, Clone, Copy)]
struct Pos {
    r: u8,
    c: u8,
}
impl Distribution<Pos> for Standard {
    fn sample<R: Rng + ?Sized>(&self, r: &mut R) -> Pos {
        Pos {
            r: r.gen_range(0, LEN) as u8,
            c: r.gen_range(0, LEN) as u8,
        }
    }
}

impl Pos {
    fn new(r: u8, c: u8) -> Pos {
        Pos { r, c }
    }
    fn next(self, dir: Dir) -> Pos {
        match dir {
            Dir::H => Pos {
                r: self.r,
                c: (self.c + 1) % LEN as u8,
            },
            Dir::V => Pos {
                r: (self.r + 1) % LEN as u8,
                c: self.c,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Dir {
    H, // horizontal <->
    V, // verticle ^
       //          |
       //          v
}

impl Distribution<Dir> for Standard {
    fn sample<R: Rng + ?Sized>(&self, r: &mut R) -> Dir {
        let b: bool = r.gen();
        if b {
            Dir::H
        } else {
            Dir::V
        }
    }
}

#[derive(Debug, Clone)]
struct Cell {
    count: usize,
    base: u8, // 0 -> 7
}

impl Cell {
    fn new() -> Cell {
        Cell { count: 0, base: 0 }
    }
    // true if it's removed
    fn remove(&mut self) -> bool {
        self.count -= 1;
        self.count == 0
    }
    fn can_set(&self, base: u8) -> bool {
        self.count == 0 || self.base == base
    }
    // true if it's new
    fn set(&mut self, base: u8) -> bool {
        assert!(self.can_set(base));
        self.count += 1;
        self.base = base;
        self.count == 1
    }
}

struct BaseCounts(Matrix<Cell>);

impl BaseCounts {
    fn new() -> BaseCounts {
        BaseCounts(Matrix(vec![vec![Cell::new(); LEN as usize]; LEN as usize]))
    }

    fn get_char(&self, pos: Pos) -> char {
        let cell = &self.0[pos];
        if cell.count == 0 {
            '.'
        } else {
            (b'a' + cell.base) as char
        }
    }

    fn add_pat(&mut self, pat: &[u8], pos: Pos, dir: Dir) -> i32 {
        let mut cur = pos;
        let mut diff = 0;
        for &base in pat {
            if self.0[cur].set(base) {
                diff -= 1;
            }
            cur = cur.next(dir);
        }
        diff
    }

    fn remove_pat(&mut self, pat: &[u8], pos: Pos, dir: Dir) -> i32 {
        let mut cur = pos;
        let mut diff = 0;
        for _ in pat {
            if self.0[cur].remove() {
                diff += 1;
            }
            cur = cur.next(dir);
        }
        diff
    }

    fn can_place(&self, pat: &[u8], pos: Pos, dir: Dir) -> bool {
        let mut cur = pos;
        for &base in pat {
            if !self.0[cur].can_set(base) {
                return false;
            }
            cur = cur.next(dir);
        }
        true
    }
}

fn to_num(score: (i32, i32)) -> f64 {
    score.0 as f64 + score.1 as f64 / 10.0
}

fn main() {
    let start = Instant::now();
    let time_limit = Duration::from_secs_f64(2.9);

    let n: u8 = scan();
    assert!(n == LEN);
    let m: usize = scan();
    let mut patterns: Vec<Vec<u8>> = Vec::new();
    for _ in 0..m {
        let s: Vec<char> = scan::<String>().chars().collect();
        let mut p = Vec::new();
        for c in s {
            p.push(c as u8 - b'a');
        }
        patterns.push(p);
    }
    let mut base_counts = BaseCounts::new();
    let mut placements: Vec<Option<(Pos, Dir)>> = vec![None; m];

    let mut rng = thread_rng();

    let mut score = (0, LEN as i32 * LEN as i32);

    loop {
        let time_ratio = start.elapsed().as_secs_f64() / time_limit.as_secs_f64();
        if time_ratio > 1.0 {
            break;
        }
        let start_temp = 1.0;
        let end_temp = 1.0 / 10.0;
        let temp = start_temp + (end_temp - start_temp) * time_ratio;

        let index = rng.gen_range(0, m);
        let pos: Pos = rng.gen();
        let dir: Dir = rng.gen();

        let pat = &patterns[index];

        // TODO: define Pos::iter(dir)

        let prev_placement = placements[index].clone();
        let mut score_diff = (1i32, 0i32);

        if let Some((pos, dir)) = placements[index] {
            score_diff.0 -= 1;

            assert!(placements[index].is_some());
            score_diff.1 += base_counts.remove_pat(pat, pos, dir);
            placements[index] = None;
        }

        if !base_counts.can_place(pat, pos, dir) {
            if let Some((pos, dir)) = prev_placement {
                assert!(placements[index].is_none());
                base_counts.add_pat(pat, pos, dir);
                placements[index] = Some((pos, dir));
            }
            continue;
        }

        assert!(placements[index].is_none());
        score_diff.1 += base_counts.add_pat(pat, pos, dir);
        placements[index] = Some((pos, dir));

        let new_score = (score.0 + score_diff.0, score.1 + score_diff.1);
        let diff = to_num(new_score) - to_num(score);
        let prob = (diff / temp).exp();
        if prob >= rng.gen::<f64>() {
            /*
            if score < new_score {
                eprintln!("{:?} {:?} {:.2}", start.elapsed(), score, to_num(score));
            }
            */
            score = new_score;
        } else {
            assert!(placements[index].is_some());
            base_counts.remove_pat(pat, pos, dir);
            placements[index] = None;

            if let Some((pos, dir)) = prev_placement {
                assert!(placements[index].is_none());
                base_counts.add_pat(pat, pos, dir);
                placements[index] = Some((pos, dir));
            }
        }
    }

    for r in 0..LEN {
        for c in 0..LEN {
            print!("{}", base_counts.get_char(Pos::new(r, c)));
        }
        print!("\n");
    }

    dbg!(score);
    dbg!(score.0 as f64 / m as f64);
}
