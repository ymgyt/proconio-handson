#![allow(unused_variables)]
#![allow(unused_mut)]

use std::marker::PhantomData;

// # ![feature(trace_macros)]
// trace_macros!(true);

lazy_static::lazy_static! {
    static ref STDIN_SOURCE: std::sync::Mutex<OnceSource<std::io::BufReader<std::io::Stdin>>> =
        std::sync::Mutex::new(OnceSource::new(std::io::BufReader::new(std::io::stdin())));
}

macro_rules! input {
    // terminator
    (@from [$source:expr] @rest) => {};

    (@from [$source:expr] @rest $($rest:tt)*) => {
        input! {
            @from [$source]
            @mut []
            @rest $($rest)*
        }
    };

    // parse variable pattern
    (@from [$source:expr] @mut [$($mut:tt)?] @rest $var:tt: $($rest:tt)*) => {
        input! {
            @from [$source]
            @mut [$($mut)*]
            @var $var
            @kind []
            @rest $($rest)*
        }
    };

    (@from [$source:expr] @mut [$($mut:tt)?] @var $var:tt @kind [$($kind:tt)*] @rest) => {
        let $($mut)* $var = read_value!(@source [$source] @kind [$($kind)*]);
    };
    (@from [$source:expr] @mut [$($mut:tt)?] @var $var:tt @kind [$($kind:tt)*] @rest, $($rest:tt)*) => {
        input!(@from [$source] @mut [$($mut)*] @var $var @kind [$($kind)*] @rest);
        input!(@from [$source] @rest $($rest)*);
    };
    (@from [$source:expr] @mut [$($mut:tt)?] @var $var:tt @kind [$($kind:tt)*] @rest $tt:tt $($rest:tt)*) => {
        input!(@from [$source] @mut [$($mut)*] @var $var @kind [$($kind)* $tt] @rest $($rest)*);
    };

    (from $source:expr, $($rest:tt)*) => {
       let mut s = $source;
       input! {
           @from [&mut s]
           @rest $($rest)*
       }
    };

    ($($rest:tt)*) => {
        let mut locked_stdin = STDIN_SOURCE.lock().unwrap();
        input! {
            @from [&mut *locked_stdin]
            @rest $($rest)*
        }
        drop(locked_stdin); // release the lock
    };
}

macro_rules! read_value {
    // array
    (@source [$source:expr] @kind [[$($kind:tt)*]]) => {
        read_value!(@array @source [$source] @kind [] @rest $($kind)*)
    };
    (@array @source [$source:expr] @kind [$($kind:tt)*] @rest) => {{
        let len = <usize as Readable>::read($source);
        read_value!(@source [$source] @kind [[$($kind)*; len]])
    }};
    (@array @source [$source:expr] @kind [$($kind:tt)*] @rest ; $($rest:tt)*) => {
        read_value!(@array @source [$source] @kind [$($kind)*] @len [$($rest)*])
    };
    (@array @source [$source:expr] @kind [$($kind:tt)*] @rest $tt:tt $($rest:tt)*) => {
        read_value!(@array @source [$source] @kind [$($kind)* $tt] @rest $($rest)*)
    };
    (@array @source [$source:expr] @kind [$($kind:tt)*] @len [$($len:tt)*]) => {{
        let len = $($len)*;
        (0..len)
            .map(|_| read_value!(@source [$source] @kind [$($kind)*]))
            .collect::<Vec<_>>()
    }};


    (@source [$source:expr] @kind [$kind:ty]) => {
        <$kind as Readable>::read($source)
    };
}

pub trait Source<R: std::io::BufRead> {
    /// Gets a whitespace-splitted next token.
    fn next_token(&mut self) -> Option<&str>;

    /// Check if tokens are empty
    fn is_empty(&mut self) -> bool;

    /// Force gets a whitespace-splitted next token.
    fn next_token_unwrap(&mut self) -> &str {
        self.next_token().expect(concat!(
        "failed to get the next token; ",
        "maybe reader reached an end of input. ",
        "ensure that arguments for `input!` macro is correctly ",
        "specified to match the problem input."
        ))
    }
}

pub trait Readable {
    type Output;

    fn read<R: std::io::BufRead, S: Source<R>>(source: &mut S) -> Self::Output;
}

impl<T: std::str::FromStr> Readable for T
    where
        T::Err: std::fmt::Debug,
{
    type Output = T;
    fn read<R: std::io::BufRead, S: Source<R>>(source: &mut S) -> T {
        let token = source.next_token_unwrap();
        match token.parse() {
            Ok(v) => v,
            Err(e) => panic!(
                concat!(
                "failed to parse the input `{input}` ",
                "to the value of type `{ty}`: {err:?}; ",
                "ensure that the input format is collectly specified ",
                "and that the input value must handle specified type.",
                ),
                input = token,
                ty = std::any::type_name::<T>(),
                err = e,
            ),
        }
    }
}

struct OnceSource<R: std::io::BufRead> {
    tokens: std::iter::Peekable<std::str::SplitWhitespace<'static>>,
    context: Box<str>,
    _read: std::marker::PhantomData<R>,
}

impl<R: std::io::BufRead> OnceSource<R> {
    fn new(mut source: R) -> OnceSource<R> {
        let mut context = String::new();
        source.read_to_string(&mut context)
            .unwrap();

        let context = context.into_boxed_str();

        let mut res = OnceSource {
            context,
            tokens: "".split_whitespace().peekable(),
            _read: PhantomData,
        };

        use std::mem;
        let context: &'static str = unsafe { mem::transmute(&*res.context) };
        res.tokens = context.split_whitespace().peekable();

        res
    }
}

impl<R: std::io::BufRead> Source<R> for OnceSource<R> {
    fn next_token(&mut self) -> Option<&str> {
        self.tokens.next()
    }

    fn is_empty(&mut self) -> bool {
        self.tokens.peek().is_none()
    }
}

impl<'a> From<&'a str> for OnceSource<std::io::BufReader<&'a [u8]>> {
    fn from(s: &'a str) -> OnceSource<std::io::BufReader<&'a [u8]>> {
        OnceSource::new(std::io::BufReader::new(s.as_bytes()))
    }
}

enum Chars {}

impl Readable for Chars {
    type Output = Vec<char>;
    fn read<R: std::io::BufRead, S: Source<R>>(source: &mut S) -> Vec<char> {
        source.next_token_unwrap().chars().collect()
    }
}

enum Bytes {}

impl Readable for Bytes {
    type Output = Vec<u8>;
    fn read<R: std::io::BufRead, S: Source<R>>(source: &mut S) -> Vec<u8> {
        source.next_token_unwrap().bytes().collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_1() {
        let source = OnceSource::from("10\n");

        input! {
            from source,
            n: u8,
        }

        assert_eq!(n, 10);
    }

    #[test]
    fn empty() {
        let source = OnceSource::from("");

        input! {
            from source,
        }
    }

    #[test]
    fn marker_chars() {
        let source = OnceSource::from("abcd");

        input! {
            from source,
            c: Chars,
        }
        ;

        assert_eq!(c, vec!['a', 'b', 'c', 'd']);
    }

    #[test]
    fn marker_bytes() {
        let source = OnceSource::from("ABC");

        input! {
            from source,
            b: Bytes,
        }
        ;

        assert_eq!(b, vec![0x41, 0x42, 0x43]);
    }

    #[test]
    fn array_1() {
        let source = OnceSource::from("1 2 3");

        input! {
        from source,
        a: [i32; 3],
    }

        assert_eq!(a, [1, 2, 3]);
    }
}

fn main() {
    let source = OnceSource::from("3 1 2 3");

    input! {
        from source,
        n: usize,
        a: [i32; n],
    }

    assert_eq!(a, [1, 2, 3]);
    println!("{:?}", a);
}

