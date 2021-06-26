mod judge;
mod text_scanner;

const LEN: u8 = 20u8;

use rand::distributions::{Distribution, Standard};
use rand::prelude::*;
use std::collections::HashSet;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Neg;
use std::time::Duration;
use std::time::Instant;
use text_scanner::scan;

use crate::judge::{compute_score_detail, Input, Output};

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

fn find_maximum_prefix(row: &str, new: &str) -> usize {
    for i in (0..=(row.len().min(new.len()))).rev() {
        if row[row.len() - i..] == new[0..i] {
            return i;
        }
    }
    unreachable!();
}

fn solve(input: &judge::Input, time_limit: Duration) -> judge::Output {
    let start = Instant::now();

    let mut m = input.M;

    let mut pattern_strs: HashSet<String> = HashSet::new();
    for i in 0..m {
        let s: String = input.s[i].iter().collect::<String>();
        pattern_strs.insert(s.clone());
    }

    let mut pattern_strs: Vec<String> = pattern_strs.into_iter().collect::<Vec<_>>();
    pattern_strs.sort_by_key(|s| s.len());
    m = pattern_strs.len();

    let mut patterns: Vec<Vec<u8>> = Vec::new();
    for s in &pattern_strs {
        let s: Vec<char> = s.chars().collect();
        let mut p = Vec::new();
        for c in s {
            p.push(c as u8 - b'A');
        }
        patterns.push(p);
    }

    // TODO: handle equivalent strings get more scores

    let mut includes: Vec<HashSet<usize>> = vec![HashSet::new(); m];
    let mut included_by: Vec<Vec<usize>> = vec![Vec::new(); m];
    for i in 0..m {
        for j in i + 1..m {
            if pattern_strs[j].contains(&pattern_strs[i]) {
                includes[j].insert(i);
                included_by[i].push(j);
            }
        }
    }

    let mut estimated_score = 0;

    let mut rng = thread_rng();

    let mut used = HashSet::new();

    let mut answer = Vec::new();
    for _ in 0..LEN {
        let first = (0..m)
            .filter(|idx| !used.contains(idx))
            .max_by_key(|idx| includes[*idx].len())
            .unwrap();

        let mut row = pattern_strs[first].to_string();
        used.insert(first);
        estimated_score += 1;

        let to_remove = includes[first].clone();
        estimated_score += to_remove.len();
        for x in to_remove {
            for &i in &included_by[x] {
                includes[i].remove(&x);
            }
        }

        while let Some((next, com_len)) = (0..m)
            .filter(|idx| !used.contains(idx))
            .map(|idx| (idx, find_maximum_prefix(&row, &pattern_strs[idx])))
            .filter(|(idx, com_len)| row.len() + pattern_strs[*idx].len() - com_len <= LEN as usize)
            .max_by_key(|&(idx, com_len)| {
                (
                    if com_len == pattern_strs[idx].len() {
                        1
                    } else {
                        0
                    },
                    com_len,
                    ((pattern_strs[idx].len() - com_len) as i32).neg(),
                    includes[idx].len(),
                )
            })
        {
            row += &pattern_strs[next][com_len..];
            assert!(row.len() <= LEN as usize);
            used.insert(next);
            estimated_score += 1;

            let to_remove = includes[next].clone();
            estimated_score += to_remove.len();
            for x in to_remove {
                for &i in &included_by[x] {
                    includes[i].remove(&x);
                }
            }
        }

        while row.len() < LEN as usize {
            let c = (b'A' + rng.gen_range(0, 8) as u8) as char;
            row.push(c);
        }

        answer.push(row.chars().collect());
    }

    let mut iteration = 0;
    while start.elapsed() <= time_limit {
        iteration += 1;
        let idx1 = rng.gen_range(0, LEN) as usize;
        let idx2 = rng.gen_range(0, LEN) as usize;
        let score = compute_score_detail(&input, &answer).0;
        answer.swap(idx1, idx2);
        let new_score = compute_score_detail(&input, &answer).0;
        if score <= new_score {
            //dbg!(new_score);
        } else {
            answer.swap(idx1, idx2);
        }
    }
    dbg!(iteration);

    answer
}

fn local_test() {
    let num = 100;
    let mut sum = 0i64;
    for seed in 0..num {
        let input = judge::gen(seed);
        let output = solve(&input, Duration::from_secs_f64(1.5));
        let (score, reason) = judge::compute_score_detail(&input, &output);
        if reason != "" {
            panic!("reason = {}", reason);
        }
        eprintln!("seed = {:02}, score = {}", seed, score,);
        sum += score;
    }
    eprintln!("{}", sum / num as i64);
}

const LOCAL: bool = true;

fn main() {
    if LOCAL {
        local_test();
        return;
    }

    let time_limit = Duration::from_secs_f64(2.9);

    let n: u8 = scan();
    assert!(n == LEN);

    let m: usize = scan();

    let mut strs = Vec::new();
    for _ in 0..m {
        let s: String = scan::<String>();
        strs.push(s.chars().collect());
    }

    let input = Input { M: m, s: strs };

    let output: Output = solve(&input, time_limit);

    for i in 0..LEN {
        println!("{}", output[i as usize].iter().collect::<String>());
    }
}
