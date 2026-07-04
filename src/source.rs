#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SrcPos {
    pub line: usize,
    pub col: usize,
    pub idx: usize,
}

impl std::fmt::Display for SrcPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl SrcPos {
    pub fn new(line: usize, col: usize, idx: usize) -> Self {
        Self { line, col, idx }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: SrcPos,
    end: SrcPos,
}

impl Span {
    pub fn new(start: SrcPos, end: SrcPos) -> Self {
        Self { start, end }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.start.line, self.start.col)
    }
}
