use std::collections::BTreeSet;
use std::fmt::{self, Debug};

const NL: char = '\n';

const EXCEED_WIDTH: usize = 120;
const MAX_WIDTH: usize = EXCEED_WIDTH - 1;
const MAX_ALIGN_COL: usize = EXCEED_WIDTH / 3;

const INDENT: &'static str = "    ";
const WRAP_INDENT: &'static str = "        ";

#[macro_export]
macro_rules! need_wrap {
    ($ts:expr, $($s:expr),+) => ({
        $ts.need_wrap(&[$($s),+])
    });
}

#[macro_export]
macro_rules! need_nl_indent {
    ($ts:expr, $($s:expr),+) => ({
        $ts.need_nl_indent(&[$($s),+])
    });
}

macro_rules! raw_insert {
    ($sf:expr, $s:expr) => ({
        $sf.s.push_str($s);

        $sf.col += $s.len();
        if $sf.col > EXCEED_WIDTH {
            $sf.exceed_lines.insert($sf.line);
        }
    });
}

macro_rules! minus_nf {
    ($a: expr, $b: expr) => ({
        if $a <= $b {
            0
        } else {
            $a - $b
        }
    })
}

#[inline]
fn list_len_info(list: &[&str]) -> (usize, usize) {
    let prefix_len = if list.len() > 1 {
        list.iter().take(list.len() - 1).map(|s| str_one_line_len(s)).sum()
    } else {
        0
    };
    let len = list.iter().map(|s| str_one_line_len(s)).sum();
    (prefix_len, len)
}

#[inline]
fn str_one_line_len(s: &str) -> usize {
    if let Some(pos) = s.find('\n') {
        pos
    } else {
        s.len()
    }
}

#[inline]
fn fill_str(ch: char, count: usize) -> String {
    let mut s = String::with_capacity(count);
    for _ in 0..count {
        s.push(ch);
    }
    s
}

#[derive(Default)]
pub struct Typesetter {
    line: u32,
    col: usize,
    indent: String,
    align_stack: Vec<usize>,

    s: String,
    exceed_lines: BTreeSet<u32>,
    trailing_ws_lines: BTreeSet<u32>,
}

pub struct TsResult {
    pub s: String,
    pub exceed_lines: BTreeSet<u32>,
    pub trailing_ws_lines: BTreeSet<u32>,
}

impl Debug for Typesetter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pos: ({}, {})\n", self.line, self.col)?;
        write!(f, "indent: \"{}\"\n", self.indent)?;
        write!(f, "align stack: ")?;
        Debug::fmt(&self.align_stack, f)?;
        write!(f, "\nexceed lines: ")?;
        Debug::fmt(&self.exceed_lines, f)
    }
}

impl Typesetter {
    pub fn new() -> Typesetter {
        Typesetter {
            line: 1,
            ..Default::default()
        }
    }

    pub fn result(self) -> TsResult {
        TsResult {
            s: self.s,
            exceed_lines: self.exceed_lines,
            trailing_ws_lines: self.trailing_ws_lines,
        }
    }

    #[inline]
    pub fn force_insert(&mut self, s: &str) {
        self.s.push_str(s);
    }

    #[inline]
    pub fn raw_insert(&mut self, s: &str) {
        raw_insert!(self, s);
    }

    #[inline]
    pub fn insert(&mut self, s: &str) {
        if need_wrap!(self, s) {
            self.wrap_insert(s);
        } else {
            self.raw_insert(s);
        }
    }

    #[inline]
    pub fn indent(&mut self) {
        self.indent.push_str(INDENT);
    }

    #[inline]
    pub fn outdent(&mut self) {
        let len = self.indent.len();
        self.indent.truncate(len - INDENT.len());
    }

    #[inline]
    pub fn insert_indent(&mut self) {
        raw_insert!(self, &self.indent);
    }

    #[inline]
    pub fn nl(&mut self) {
        if let Some(ch) = self.s.chars().last() {
            if ch != NL && ch.is_whitespace() {
                self.trailing_ws_lines.insert(self.line);
            }
        }

        self.s.push(NL);
        self.line += 1;
        self.col = 0;
    }

    #[inline]
    pub fn nl_indent(&mut self) {
        self.nl();
        self.insert_indent();
    }

    #[inline]
    pub fn can_one_line(&self, s: &str) -> bool {
        self.left() > s.len()
    }

    #[inline]
    pub fn need_wrap(&self, list: &[&str]) -> bool {
        let (prefix_len, len) = list_len_info(list);
        self.need_wrap_len(prefix_len, len)
    }

    #[inline]
    pub fn need_nl_indent(&self, list: &[&str]) -> bool {
        let (prefix_len, len) = list_len_info(list);
        self.need_nl_indent_len(prefix_len, len)
    }

    #[inline]
    pub fn wrap(&mut self) {
        self.nl();

        if self.should_align() {
            self.insert_align();
        } else {
            self.insert_indent();
            self.insert_wrap_indent();
        }
    }

    #[inline]
    pub fn insert_mark_align(&mut self, s: &str) {
        self.raw_insert(s);
        self.mark_align();
    }

    #[inline]
    pub fn insert_unmark_align(&mut self, s: &str) {
        self.raw_insert(s);
        self.unmark_align();
    }

    #[inline]
    fn wrap_insert(&mut self, s: &str) {
        self.wrap();
        self.raw_insert(s);
    }

    #[inline]
    fn need_wrap_len(&self, prefix_len: usize, len: usize) -> bool {
        (minus_nf!(self.left(), prefix_len) <= 0) || (len > self.left() && len <= self.nl_left())
    }

    #[inline]
    fn nl_left(&self) -> usize {
        if self.should_align() {
            self.nl_align_left()
        } else {
            self.nl_wrap_left()
        }
    }

    #[inline]
    fn should_align(&self) -> bool {
        match self.align_stack.last() {
            Some(col) if *col <= MAX_ALIGN_COL => true,
            _ => false,
        }
    }

    #[inline]
    fn nl_align_left(&self) -> usize {
        minus_nf!(MAX_WIDTH, *self.align_stack.last().unwrap())
    }

    #[inline]
    fn nl_wrap_left(&self) -> usize {
        minus_nf!(MAX_WIDTH, self.indent.len() + WRAP_INDENT.len())
    }

    #[inline]
    fn need_nl_indent_len(&self, prefix_len: usize, len: usize) -> bool {
        (minus_nf!(self.left(), prefix_len) <= 0) || (len > self.left() && len <= self.nl_indent_left())
    }

    #[inline]
    fn left(&self) -> usize {
        minus_nf!(MAX_WIDTH, self.col)
    }

    #[inline]
    fn nl_indent_left(&self) -> usize {
        minus_nf!(MAX_WIDTH, self.indent.len())
    }

    #[inline]
    fn insert_wrap_indent(&mut self) {
        self.raw_insert(WRAP_INDENT);
    }

    #[inline]
    fn insert_align(&mut self) {
        let blank = fill_str(' ', *self.align_stack.last().unwrap());
        self.raw_insert(&blank);
    }

    #[inline]
    fn mark_align(&mut self) {
        self.align_stack.push(self.col);
    }

    #[inline]
    fn unmark_align(&mut self) {
        self.align_stack.pop();
    }
}
