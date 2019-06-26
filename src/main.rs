extern crate termcolor;
#[macro_use] extern crate errer;
#[macro_use] extern crate errer_derive;

extern crate lazy_static;
extern crate structopt;
extern crate cpython;

use errer::PrintError;
use lazy_static::lazy_static;
use structopt::StructOpt;
use termcolor::*;
use cpython::*;

use std::sync::Mutex;
use std::io;
use std::io::Write;

#[derive(Errer)]
enum Error {
    #[errer(from)]
    Py(PyErr),
    EmptyInput,
    InvalidKey(char),
    BadIndex(usize),
    NoSeparators(usize)
}

impl PrintError for Error {
    fn print(&self, io: &mut StandardStream) -> io::Result<()> {
        io.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;

        match self {
            Error::EmptyInput => {
                writeln!(io, "cannot decode nothing. give me some input")?;
            },
            Error::InvalidKey(key) => {
                write!(io, "error decoding: ")?;
                io.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
                writeln!(io, "keys (the first letter) must be an encodable character. '{}' is not part of \"{}\"", key, &ENCODE_PATTERN)?;
            },
            Error::BadIndex(key) | Error::NoSeparators(key) => {
                write!(io, "error decoding (key {}): ", key)?;
                io.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            
                match self {
                    Error::BadIndex(_) => {
                        writeln!(io, "invalid index")?;
                    },
                    Error::NoSeparators(_) => {
                        writeln!(io, "no separators found")?;
                    },
                    _ => ()
                }

                io.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                writeln!(io, "maybe try using a different key.")?;
            },
            Error::Py(pyerr) => {
                writeln!(io, "error while interpreting python")?;
                io.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                writeln!(io, "report this stacktrace / try to debug it yourself. this is likely platform-specific")?;

                io.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
                with_py(|py| pyerr.clone_ref(py).print(py));
            }
        }

        io.set_color(ColorSpec::new().set_fg(Some(Color::White)))
    }
}

res!(Error);

#[derive(Default)]
struct State {
    variance: usize,
    key: Option<usize>,
    keyboard: Option<PyModule>,
    lens: Vec<usize>
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State::default());
}

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "v", long = "variance", default_value = "8")]
    variance: usize,
    #[structopt(name = "DECODE")]
    decode: Option<String>,
}

const ENCODE_PATTERN: &str = "1234567890qwertyuiopasdfghjklzxcvbnm,.;'[]";
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

fn wait() {
    std::thread::sleep(std::time::Duration::from_millis(1)); //give 10ms for key to register in target application
}

fn with_py<T, F: FnOnce(Python<'_>) -> T>(f: F) -> T {
    let gil = Python::acquire_gil();
    let py = gil.python();

    f(py)
}

fn callback(py: Python, event: PyObject) -> PyResult<bool> {
    let name = event.getattr(py, "name").unwrap().extract::<String>(py).unwrap();
    
    py.allow_threads(move || {
        let mut state = STATE.lock().unwrap();

        let c = match name.as_str() {
            x if x.len() == 1 => x.chars().next().unwrap(),
            "enter" => {
                let shift = with_py(|py| state.keyboard.as_ref().unwrap()
                    .call(py, "is_pressed", ("shift",), None).unwrap()
                    .extract::<bool>(py).unwrap());
                
                if !shift {
                    state.key = None;
                }

                return;
            },
            "space" => ' ',
            "backspace" => {
                if let Some(x) = state.lens.pop() {
                    let keyboard = state.keyboard.as_ref().unwrap();

                    for _ in 0..x-1 { //since backspace has already been done once, we use x-1
                        wait();
                        with_py(|py| keyboard.call(py, "send", ("backspace",), None).unwrap());
                    }
                }

                return;
            },
            _ => return
        };

        let c_k = match DECODE_PATTERN.find(c) {
            Some(k) => k,
            None => return
        };

        let mut keys = expand_base(state.variance, c_k);
    
        let key = match state.key {
            Some(k) => k,
            None => match ENCODE_PATTERN.find(c) {
                Some(k) => {
                    keys.insert(0, 0);
                
                    state.key = Some(k);
                    k
                },
                None => return
            }
        };

        keys.push(state.variance+1+(c_k%(state.variance-1)));
        state.lens.push(keys.len());
        if state.lens.len() > 50 {
            state.lens.remove(0);
        }
        
        wait();
        
        let keyboard = state.keyboard.as_ref().unwrap();
        with_py(|py| {
            keyboard.call(py, "send", ("backspace",), None).unwrap();
        });
        
        for x in keys {
            let x = (x+key)%ENCODE_PATTERN.len();
            let x = &ENCODE_PATTERN[x..x+1];

            wait();
            
            with_py(|py| {
                keyboard.call(py, "send", (&x,), None).unwrap();
            });
        }
    });

    Ok(true)
}

fn main_res() -> Res<()> {
    let opt = Opt::from_args();

    if let Some(decode) = opt.decode {
        let mut s = String::new();

        if decode.len() == 0 {
            return Err(Error::EmptyInput);
        }
        
        let mut iter = decode.chars();
        let key_char = iter.next().unwrap();
        let key = get_k(key_char).ok_or(Error::InvalidKey(key_char))?;
        
        let mut chunk = Vec::new();
        
        for x in iter {
            let k = match get_k(x) {
                Some(k) => k, None => {
                    s.push(x);
                    continue;
                }
            };

            let x = modulo(k as i32-key as i32, ENCODE_PATTERN.len() as i32) as usize;

            if x > opt.variance {
                let i = compress_base(opt.variance, chunk.clone());

                if let Some(x) = DECODE_PATTERN.get(i..i+1) {
                    s.push_str(&x);
                } else {
                    return Err(Error::BadIndex(key));
                }
                
                chunk.clear();
            } else {
                chunk.push(x);
            }
        }

        if s.len() > 0 {
            println!("{}", s);
        } else {
            return Err(Error::NoSeparators(key));
        }

        return Ok(());
    }

    let gil = Python::acquire_gil();
    let py = gil.python();
    
    let keyboard_wait;

    {
        let mut l = STATE.lock().unwrap();
        l.variance = opt.variance;

        
        let keyboard = py.import("keyboard")?;
        
        let cb = py_fn!(py, callback(event: PyObject));
        keyboard.call(py, "on_press", (cb,), None)?;

        keyboard_wait = keyboard.get(py, "wait")?;
        l.keyboard = Some(keyboard);
    }

    if let Err(x) = keyboard_wait.call(py, NoArgs, None) {
        if x.get_type(py) == cpython::exc::KeyboardInterrupt::type_object(py) {
            return Ok(());
        } else {
            return Err(x.into());
        }
    }

    Ok(())
}

fn main() {
    if let Err(x) = main_res() {
        x.print(&mut StandardStream::stderr(ColorChoice::Auto)).unwrap();
    }
}