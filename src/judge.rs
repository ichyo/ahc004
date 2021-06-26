#![allow(non_snake_case, dead_code, unused_imports, unused_macros)]

use rand::prelude::*;
pub trait SetMinMax {
    fn setmin(&mut self, v: Self) -> bool;
    fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn setmin(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn setmax(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}
pub const N: usize = 20;

pub type Output = Vec<Vec<char>>;

pub struct Input {
    pub M: usize,
    pub s: Vec<Vec<char>>,
}

pub const DIR: [(usize, usize); 2] = [(0, 1), (1, 0)];

pub fn mod_n(i: usize) -> usize {
    if i >= N {
        i - N
    } else {
        i
    }
}

pub fn is_substring(a: &Vec<Vec<char>>, b: &Vec<char>, i: usize, j: usize, d: usize) -> bool {
    let (di, dj) = DIR[d];
    for k in 0..b.len() {
        let i = mod_n(i + di * k);
        let j = mod_n(j + dj * k);
        if a[i][j] != b[k] {
            return false;
        }
    }
    true
}

pub fn get_substring(a: &Vec<Vec<char>>, i: usize, j: usize, d: usize, k: usize) -> Vec<char> {
    let (di, dj) = DIR[d];
    let mut b = vec![];
    for k in 0..k {
        let i = mod_n(i + di * k);
        let j = mod_n(j + dj * k);
        b.push(a[i][j]);
    }
    b
}

pub fn compute_score_detail(input: &Input, out: &Output) -> (i64, String) {
    let mut c = 0;
    let mut d = 0;
    for i in 0..N {
        if out[i].len() != N {
            return (0, format!("illegal length: {}", out[i].len()));
        }
        for j in 0..N {
            if (out[i][j] < 'A' || 'H' < out[i][j]) && out[i][j] != '.' {
                return (0, format!("illegal char: {}", out[i][j]));
            }
            if out[i][j] == '.' {
                d += 1;
            }
        }
    }
    for k in 0..input.M {
        let mut used = false;
        'find: for i in 0..N {
            for j in 0..N {
                for d in 0..2 {
                    if is_substring(&out, &input.s[k], i, j, d) {
                        used = true;
                        break 'find;
                    }
                }
            }
        }
        if used {
            c += 1;
        }
    }
    let score = if c < input.M {
        1e8 * c as f64 / input.M as f64
    } else {
        1e8 * (2 * N * N) as f64 / (2 * N * N - d) as f64
    };
    (score.round() as i64, String::new())
}

pub fn gen(seed: u64) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    let mut a = mat!['.'; N; N];
    for i in 0..N {
        for j in 0..N {
            a[i][j] = (b'A' + rng.gen_range(0, 8)) as char;
        }
    }
    let L = rng.gen_range(4, 11);
    let M = rng.gen_range(400, 801) as usize;
    let mut s = vec![];
    for _ in 0..M {
        let i = rng.gen_range(0, N as u32) as usize;
        let j = rng.gen_range(0, N as u32) as usize;
        let d = rng.gen_range(0, 2) as usize;
        let k = rng.gen_range(L - 2, L + 3) as usize;
        s.push(get_substring(&a, i, j, d, k));
    }
    Input { M, s }
}
