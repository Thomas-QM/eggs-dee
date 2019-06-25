const ENCODE_PATTERN: &str = " 1234567890qwertyuiopasdfghjklzxcvbnm,<.>/?";
const DECODE_PATTERN: &str = " QWERTYUIOPASDFGHJKLZXCVBNMqwertyuiopasdfghjkl1234567890-_=+;:'\"zxcvbnm,<.>/?";

// https://stackoverflow.com/questions/14997165/fastest-way-to-get-a-positive-modulo-in-c-c
fn modulo(i: i32, n: i32) -> i32 {
	(i % n + n) % n
}

fn expand_base(to_base: usize, mut i: usize) -> Vec<usize> {
	let mut vec = Vec::new();

	loop {
		let x = i/to_base;
		let rem = i%to_base;

		if x >= 1 {
			vec.push(rem);

			i = x;
		} else {
			vec.push(rem);
			
			break;
		}
	}

	vec.reverse();
	vec
}

fn compress_base(from_base: usize, i: Vec<usize>) -> usize {
	let l = i.len();
	i.into_iter().enumerate().map(|(i, num)| num*from_base.pow((l-(i+1)) as u32)).sum()
}

fn get_k(c: char) -> Option<usize> {
	ENCODE_PATTERN.find(c)
}

#[no_mangle]
fn decode(variance: usize, decode: &str) -> String {
	let mut s = String::new();
		
	let mut iter = decode.chars();
	let key = get_k(iter.next().unwrap()).unwrap();
	
	let mut chunk = Vec::new();
	
	for x in iter {
		let k = match get_k(x) {
			Some(k) => k, None => {
				s.push(x);
				continue;
			}
		};

		if k > key+variance || (k > ((key+variance)%ENCODE_PATTERN.len()) && k < key) {
			let i = compress_base(variance, chunk.clone());

			s.push_str(&DECODE_PATTERN[i..i+1]);
			
			chunk.clear();
		} else {
			let x = modulo(k as i32-key as i32, ENCODE_PATTERN.len() as i32) as usize;
			
			chunk.push(x);
		}
	}

	s
}

fn encode_char<B: FnMut(), W: FnMut(&str)>(variance: usize, key: &mut Option<usize>, c: char, mut backspace: B, mut write: W) {
	let c_k = match DECODE_PATTERN.find(c) {
		Some(k) => k,
		None => return
	};
	
	let mut keys = expand_base(variance, c_k);
	
	let key = match key {
		None => match ENCODE_PATTERN.find(c) {
			Some(k) => {
				*key = Some(k);
				keys.insert(0, 0);

				k
			},
			None => return
		},
		Some(k) => *k
	};

	backspace();

	keys.push(variance+(1+(c_k%(variance-1))));

	for mut x in keys {
		x = (x+key)%ENCODE_PATTERN.len();
		write(&ENCODE_PATTERN[x..x+1]);
	}
}

#[no_mangle]
fn encode(variance: usize, s: &str) -> String {
	let mut key = None;
	let mut out = String::new();

	for c in s.chars() {
		encode_char(variance, &mut key, c, || {}, |s| out.push_str(s))
	}

	out
}

#[no_mangle]
fn run(variance: usize, get: fn() -> *const [u16], backspace: fn(), write: fn(*const [u16])) {
	let mut key = None;

	loop {
		let s = String::from_utf16_lossy(unsafe { get().as_ref().unwrap() });

		if s.len() > 0 {
			let c = s.chars().next().unwrap();

			encode_char(variance, &mut key, c, backspace, |s| {
				let x: Vec<u16> = s.encode_utf16().collect();
				write(&*x)
			});
		}
	}
}