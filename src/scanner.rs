use std::str::from_utf8;

pub struct Scanner<'a> {
    source: &'a [u8],
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner<'_> {
    pub fn new(source: &[u8]) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }
}
