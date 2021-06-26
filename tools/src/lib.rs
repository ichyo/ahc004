#![allow(non_snake_case, dead_code, unused_imports, unused_macros)]

use rand::prelude::*;
use proconio::{input, marker::*, source::*};
use svg::node::element::{Rectangle, Circle, Path, path::Data};

pub trait SetMinMax {
	fn setmin(&mut self, v: Self) -> bool;
	fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T where T: PartialOrd {
	fn setmin(&mut self, v: T) -> bool {
			*self > v && { *self = v; true }
	}
	fn setmax(&mut self, v: T) -> bool {
			*self < v && { *self = v; true }
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

impl std::fmt::Display for Input {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{} {}", N, self.M)?;
		for i in 0..self.M {
			writeln!(f, "{}", self.s[i].iter().collect::<String>())?;
		}
		Ok(())
	}
}

pub fn parse_input(f: &str) -> Input {
	let mut f = proconio::source::once::OnceSource::from(f);
	input! {
		from &mut f,
		_: usize, M: usize,
		s: [Chars; M]
	}
	Input { M, s }
}

pub fn parse_output(_input: &Input, f: &str) -> Output {
	let f = proconio::source::once::OnceSource::from(f);
	input! {
		from f,
		out: [Chars; N]
	}
	out
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
		for i in 0..N {
			for j in 0..N {
				for d in 0..2 {
					if is_substring(&out, &input.s[k], i, j, d) {
						used = true;
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

/// 0 <= val <= 1
fn color(val: f64) -> String {
	let x = val * 2.0 - 1.0;
	let f = |d| (255.0 / (1.0 + f64::exp(-15.0 * (x + d)))).round() as i32;
	let r = f(-0.2) * 5 / 6;
	let g = (f(0.6) - f(-0.6)) * 2 / 3;
	let b = (255 - f(0.2)) * 11 / 12;
	format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn rect(x: usize, y: usize, w: usize, h: usize, fill: &str) -> Rectangle {
	Rectangle::new().set("x", x).set("y", y).set("width", w).set("height", h).set("fill", fill)
}

fn text(x: usize, y: usize, size: usize, s: &str) -> svg::node::element::Text {
	svg::node::element::Text::new().set("x", x).set("y", y).set("font-size", size).set("text-anchor", "middle").add(svg::node::Text::new(s))
}

pub fn vis_default(input: &Input, out: &Output) -> (i64, String, String) {
	vis(input, out, -1)
}

pub fn vis(input: &Input, out: &Output, show_i: i32) -> (i64, String, String) {
	let (score, err) = compute_score_detail(input, out);
	let mut doc = svg::Document::new().set("viewBox", (0, 0, 30 * N, 30 * N)).set("width", 30 * N).set("height", 30 * N);
	doc = doc.add(rect(0, 0, 30 * N, 30 * N, "white"));
	for i in 0..=N {
		let data = Data::new().move_to((i * 30, 0)).line_by((0, N * 30));
		let path = Path::new().set("stroke", "black").set("stroke-width", 1).set("d", data);
		doc = doc.add(path);
		let data = Data::new().move_to((0, i * 30)).line_by((N * 30, 0));
		let path = Path::new().set("stroke", "black").set("stroke-width", 1).set("d", data);
		doc = doc.add(path);
	}
	let mut count_h = mat![0; N; N];
	let mut count_v = mat![0; N; N];
	for i in 0..N {
		for j in 0..N {
			for k in 0..input.M {
				if show_i == -1 || show_i as usize == k {
					for d in 0..2 {
						if is_substring(&out, &input.s[k], i, j, d) {
							let (di, dj) = DIR[d];
							for p in 0..input.s[k].len() {
								let i = mod_n(i + di * p);
								let j = mod_n(j + dj * p);
								if d == 0 {
									count_h[i][j] += 1;
								} else {
									count_v[i][j] += 1;
								}
							}
						}
					}
				}
			}
		}
	}
	for i in 0..N {
		for j in 0..N {
			if count_h[i][j] > 0 {
				let data = Data::new().move_to((j * 30, i * 30 + 15)).line_by((30, 0));
				let path = Path::new().set("stroke", "dodgerblue").set("stroke-width", (count_h[i][j] * if show_i < 0 { 1 } else { 5 }).min(20)).set("d", data);
				doc = doc.add(path);
			}
			if count_v[i][j] > 0 {
				let data = Data::new().move_to((j * 30 + 15, i * 30)).line_by((0, 30));
				let path = Path::new().set("stroke", "dodgerblue").set("stroke-width", (count_v[i][j] * if show_i < 0 { 1 } else { 5 }).min(20)).set("d", data);
				doc = doc.add(path);
			}
		}
	}
	for i in 0..N {
		for j in 0..N {
			if out[i][j] != '.' {
				doc = doc.add(text(j * 30 + 15, i * 30 + 22, 25, &out[i][j].to_string()));
			}
		}
	}
	(score, doc.to_string(), err)
}
