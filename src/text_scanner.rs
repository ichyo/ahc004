#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    EncodingError(std::string::FromUtf8Error),
    ParseError(String),
    Eof,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::IoError(ref e) => writeln!(f, "IO Error: {}", e),
            Error::EncodingError(ref e) => writeln!(f, "Encoding Error: {}", e),
            Error::ParseError(ref e) => writeln!(f, "Parse Error: {}", e),
            Error::Eof => writeln!(f, "EOF"),
        }
    }
}

impl std::error::Error for Error {}

pub fn read_line() -> Option<String> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    fread_line(&mut stdin).expect("IO error")
}

pub fn scan<T: FromTokens>() -> T {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    fscan(&mut stdin).expect("IO error")
}

pub fn scanln<T: FromTokens>() -> T {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    fscanln(&mut stdin).expect("IO error")
}

pub fn scan_iter<T: FromTokens>() -> ScanIter<T> {
    ScanIter {
        item_type: std::marker::PhantomData,
    }
}

pub fn scanln_iter<T: FromTokens>() -> ScanlnIter<T> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let s = fread_line(&mut stdin)
        .expect("IO error")
        .unwrap_or_else(String::new);
    ScanlnIter {
        cursor: std::io::Cursor::new(s),
        item_type: std::marker::PhantomData,
    }
}

pub fn fread_line<R: std::io::BufRead>(r: &mut R) -> Result<Option<String>, std::io::Error> {
    let mut buf = String::new();
    let length = r.read_line(&mut buf)?;
    if let Some('\n') = buf.chars().last() {
        buf.pop();
    }
    if let Some('\r') = buf.chars().last() {
        buf.pop();
    }
    if length == 0 {
        Ok(None)
    } else {
        Ok(Some(buf))
    }
}

pub fn fscan<R: std::io::Read, T: FromTokens>(reader: &mut R) -> Result<T, Error> {
    let mut tokenizer = Tokenizer::new(reader);
    FromTokens::from_tokens(&mut tokenizer)
}

pub fn fscanln<R: std::io::BufRead, T: FromTokens>(reader: &mut R) -> Result<T, Error> {
    let s = match fread_line(reader) {
        Ok(Some(s)) => s,
        Ok(None) => return Err(Error::Eof),
        Err(e) => return Err(Error::IoError(e)),
    };
    let mut bytes = s.as_bytes();
    let mut tokenizer = Tokenizer::new(&mut bytes);
    FromTokens::from_tokens(&mut tokenizer)
}

pub fn fscan_iter<R: std::io::Read, T: FromTokens>(reader: &mut R) -> FscanIter<R, T> {
    FscanIter {
        tokenizer: Tokenizer::new(reader),
        item_type: std::marker::PhantomData,
    }
}

pub fn fscanln_iter<R: std::io::BufRead, T: FromTokens>(
    reader: &mut R,
) -> Result<ScanlnIter<T>, Error> {
    let s = match fread_line(reader) {
        Ok(Some(s)) => s,
        Ok(None) => "".to_string(),
        Err(e) => return Err(Error::IoError(e)),
    };
    Ok(ScanlnIter {
        cursor: std::io::Cursor::new(s),
        item_type: std::marker::PhantomData,
    })
}

pub struct ScanIter<T>
where
    T: FromTokens,
{
    item_type: std::marker::PhantomData<T>,
}

impl<T: FromTokens> Iterator for ScanIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        let mut tokenizer = Tokenizer::new(&mut stdin);
        match FromTokens::from_tokens(&mut tokenizer) {
            Err(Error::Eof) => None,
            r => Some(r.expect("IO error")),
        }
    }
}

pub struct FscanIter<'a, R, T>
where
    R: std::io::Read + 'a,
    T: FromTokens,
{
    tokenizer: Tokenizer<'a, R>,
    item_type: std::marker::PhantomData<T>,
}

impl<'a, R: std::io::Read, T: FromTokens> Iterator for FscanIter<'a, R, T> {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match FromTokens::from_tokens(&mut self.tokenizer) {
            Err(Error::Eof) => None,
            r => Some(r),
        }
    }
}

pub struct ScanlnIter<T>
where
    T: FromTokens,
{
    cursor: std::io::Cursor<String>,
    item_type: std::marker::PhantomData<T>,
}

impl<'a, T: FromTokens> Iterator for ScanlnIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tokenizer = Tokenizer::new(&mut self.cursor);
        match FromTokens::from_tokens(&mut tokenizer) {
            Err(Error::Eof) => None,
            r => Some(r.expect("IO error")),
        }
    }
}

pub trait FromTokens
where
    Self: Sized,
{
    fn from_tokens(
        tokenizer: &mut dyn Iterator<Item = Result<String, Error>>,
    ) -> Result<Self, Error>;
}

macro_rules! from_tokens_primitives {
    ($($t:ty),*) => { $(
        impl FromTokens for $t {
            fn from_tokens(tokenizer: &mut dyn Iterator<Item = Result<String, Error>>) -> Result<Self, Error> {
                let token = tokenizer.next();
                match token {
                    Some(s) => s?
                        .parse::<$t>()
                        .map_err(|e| Error::ParseError(format!("{}", e))),
                    None => Err(Error::Eof),
                }
            }
        }
    )* }
}

from_tokens_primitives! {
    String,
    bool,
    f32,
    f64,
    isize,
    i8,
    i16,
    i32,
    i64,
    usize,
    u8,
    u16,
    u32,
    u64
}

impl<T1, T2> FromTokens for (T1, T2)
where
    T1: FromTokens,
    T2: FromTokens,
{
    fn from_tokens(
        tokenizer: &mut dyn Iterator<Item = Result<String, Error>>,
    ) -> Result<Self, Error> {
        Ok((T1::from_tokens(tokenizer)?, T2::from_tokens(tokenizer)?))
    }
}

impl<T1, T2, T3> FromTokens for (T1, T2, T3)
where
    T1: FromTokens,
    T2: FromTokens,
    T3: FromTokens,
{
    fn from_tokens(
        tokenizer: &mut dyn Iterator<Item = Result<String, Error>>,
    ) -> Result<Self, Error> {
        Ok((
            T1::from_tokens(tokenizer)?,
            T2::from_tokens(tokenizer)?,
            T3::from_tokens(tokenizer)?,
        ))
    }
}

impl<T1, T2, T3, T4> FromTokens for (T1, T2, T3, T4)
where
    T1: FromTokens,
    T2: FromTokens,
    T3: FromTokens,
    T4: FromTokens,
{
    fn from_tokens(
        tokenizer: &mut dyn Iterator<Item = Result<String, Error>>,
    ) -> Result<Self, Error> {
        Ok((
            T1::from_tokens(tokenizer)?,
            T2::from_tokens(tokenizer)?,
            T3::from_tokens(tokenizer)?,
            T4::from_tokens(tokenizer)?,
        ))
    }
}

impl<T1, T2, T3, T4, T5> FromTokens for (T1, T2, T3, T4, T5)
where
    T1: FromTokens,
    T2: FromTokens,
    T3: FromTokens,
    T4: FromTokens,
    T5: FromTokens,
{
    fn from_tokens(
        tokenizer: &mut dyn Iterator<Item = Result<String, Error>>,
    ) -> Result<Self, Error> {
        Ok((
            T1::from_tokens(tokenizer)?,
            T2::from_tokens(tokenizer)?,
            T3::from_tokens(tokenizer)?,
            T4::from_tokens(tokenizer)?,
            T5::from_tokens(tokenizer)?,
        ))
    }
}

impl<T1, T2, T3, T4, T5, T6> FromTokens for (T1, T2, T3, T4, T5, T6)
where
    T1: FromTokens,
    T2: FromTokens,
    T3: FromTokens,
    T4: FromTokens,
    T5: FromTokens,
    T6: FromTokens,
{
    fn from_tokens(
        tokenizer: &mut dyn Iterator<Item = Result<String, Error>>,
    ) -> Result<Self, Error> {
        Ok((
            T1::from_tokens(tokenizer)?,
            T2::from_tokens(tokenizer)?,
            T3::from_tokens(tokenizer)?,
            T4::from_tokens(tokenizer)?,
            T5::from_tokens(tokenizer)?,
            T6::from_tokens(tokenizer)?,
        ))
    }
}

struct Tokenizer<'a, R: std::io::Read + 'a> {
    reader: &'a mut R,
}

impl<'a, R: std::io::Read> Tokenizer<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        Tokenizer { reader }
    }

    pub fn next_token(&mut self) -> Result<Option<String>, Error> {
        use std::io::Read;
        let mut token = Vec::new();
        for b in self.reader.by_ref().bytes() {
            let b = b.map_err(Error::IoError)?;
            match (u8::is_ascii_whitespace(&b), token.is_empty()) {
                (false, _) => token.push(b),
                (true, false) => break,
                (true, true) => {}
            }
        }
        if token.is_empty() {
            return Ok(None);
        }
        String::from_utf8(token)
            .map(Some)
            .map_err(Error::EncodingError)
    }
}

impl<'a, R: std::io::Read> Iterator for Tokenizer<'a, R> {
    type Item = Result<String, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Some(s)) => Some(Ok(s)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
