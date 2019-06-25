extern crate structopt;
extern crate multiinput;
extern crate autopilot;

use structopt::StructOpt;
use multiinput::*;
use autopilot::key::*;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "v", long = "variance", default_value = "8")]
    variance: usize,
    #[structopt(name = "DECODE")]
    decode: Option<String>
}

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

macro_rules! match_id {
    (match $x: ident; $($id: ident),*) => {
        match $x { $(KeyId::$id => stringify!($id)),*, _ => "" }
    };
}

fn main() {
    let mut im = RawInputManager::new().unwrap();
    im.register_devices(DeviceType::Keyboards);

    let mut shift = false;
    let mut caps = false;

    let mut len = 0;
    let mut key = 0;

    let opt = Opt::from_args();

    if let Some(decode) = opt.decode {
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

            if k > key+opt.variance || (k > ((key+opt.variance)%ENCODE_PATTERN.len()) && k < key) {
                let i = compress_base(opt.variance, chunk.clone());

                s.push_str(&DECODE_PATTERN[i..i+1]);
                
                chunk.clear();
            } else {
                let x = modulo(k as i32-key as i32, ENCODE_PATTERN.len() as i32) as usize;
                
                chunk.push(x);
            }
        }

        println!("{}", s);

        return;
    }

    loop {
        if let Some(ev) = im.get_event() {
            match ev {
                RawEvent::KeyboardEvent(_, KeyId::Shift, State::Released) => shift = false,
                RawEvent::KeyboardEvent(_, KeyId::CapsLock, State::Released) => caps = false,
                RawEvent::KeyboardEvent(_, keyid, State::Pressed) => {
                    let upper = if caps { !shift } else { shift };
                    
                    match keyid {
                        KeyId::Shift => shift = true,
                        KeyId::CapsLock => caps = true,

                        x => {
                            let s = match x {
                                KeyId::Zero => "0", KeyId::One => "1", KeyId::Two => "2", KeyId::Three => "3", KeyId::Four => "4", KeyId::Five => "5", KeyId::Six => "6", KeyId::Seven => "7", KeyId::Eight => "8", KeyId::Nine => "9", KeyId::Space => " ",
                                KeyId::FullStop if upper => ">", KeyId::FullStop => ".", KeyId::Comma if upper => "<", KeyId::Comma => ",",
                                KeyId::ForwardSlash if upper => "?", KeyId::ForwardSlash => "/",
                                x => match_id!(match x; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)
                            };

                            if s.len() > 0 {
                                let mut c = s.chars().next().unwrap();
                                if !upper {
                                    c.make_ascii_lowercase();
                                }

                                let c_k = match DECODE_PATTERN.find(c) {
                                    Some(k) => k,
                                    None => continue
                                };
                               
                                let mut keys = expand_base(opt.variance, c_k);
                                
                                if len == 0 || len > 50 {
                                    match ENCODE_PATTERN.find(c) {
                                        Some(k) => key = k,
                                        None => continue
                                    }

                                    len = 0;

                                    keys.insert(0, 0);
                                }

                                len += 1;
 

                                tap(&Code(KeyCode::Backspace), &[], 0);

                                keys.push(opt.variance+(1+(c_k%(opt.variance-1))));

                                for mut x in keys {
                                    x = (x+key)%ENCODE_PATTERN.len();

                                    let x = &ENCODE_PATTERN[x..x+1];
                                    type_string(&x, &[], 0.0, 0.0);
                                }
                            }
                        }
                    }
                },
                _ => ()
            }
        } else {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}