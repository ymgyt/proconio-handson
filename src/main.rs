use std::marker::PhantomData;

// #![feature(trace_macros)]
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
}

fn main() {
    // stdin: 10\n-20\n

    input!{
        a: u8,
        b: i8,
    }

    assert_eq!(a, 10);
    assert_eq!(b, -20);
    println!("a: {}, b: {}", a,b);
}

