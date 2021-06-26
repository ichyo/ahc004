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
    fn remove(&mut self) {
        self.count -= 1;
    }
    fn can_set(&self, base: u8) -> bool {
        self.count == 0 || self.base == base
    }
    fn set(&mut self, base: u8) {
        assert!(self.can_set(base));
        self.count += 1;
        self.base = base;
    }
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
    let mut matrix = Matrix(vec![vec![Cell::new(); LEN as usize]; LEN as usize]); // (count, char)
    let mut state: Vec<Option<(Pos, Dir)>> = vec![None; m];

    let mut rng = thread_rng();

    let mut score = 0;

    while start.elapsed() <= time_limit {
        let index = rng.gen_range(0, m);
        let pos: Pos = rng.gen();
        let dir: Dir = rng.gen();

        // TODO: define Pos::iter(dir)

        let prev_state = state[index].clone();

        if let Some((pos, dir)) = state[index] {
            let mut cur = pos;
            for _ in &patterns[index] {
                matrix[cur].remove();
                cur = cur.next(dir);
            }
            assert!(state[index].is_some());
            score -= 1;
            state[index] = None;
        }

        let mut cur = pos;
        let mut valid = true;
        for &base in &patterns[index] {
            valid &= matrix[cur].can_set(base);
            cur = cur.next(dir);
        }

        if valid {
            let mut cur = pos;
            for &base in &patterns[index] {
                matrix[cur].set(base);
                cur = cur.next(dir);
            }
            assert!(state[index].is_none());
            score += 1;
            state[index] = Some((pos, dir));
        } else {
            if let Some((pos, dir)) = prev_state {
                let mut cur = pos;
                for &base in &patterns[index] {
                    matrix[cur].set(base);
                    cur = cur.next(dir);
                }
                assert!(state[index].is_none());
                score += 1;
                state[index] = prev_state;
            }
        }
    }

    for r in 0..LEN {
        for c in 0..LEN {
            let cell = &matrix[Pos::new(r, c)];
            if cell.count == 0 {
                print!(".");
            } else {
                print!("{}", (b'a' + cell.base) as char);
            }
        }
        print!("\n");
    }

    dbg!(score);
}
