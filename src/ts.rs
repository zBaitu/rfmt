use std::collections::BTreeSet;
use std::fmt::{self, Debug};

use zbase::zstr;

const NL: &'static str = "\n";

const MAX_WIDTH: usize = 100;
const MAX_ALIGN_COL: usize = 50;

const INDENT: &'static str = "    ";
const WRAP_INDENT: &'static str = "        ";

pub struct Typesetter {
    line: u32,
    col: usize,
    indent: String,
    align_stack: Vec<usize>,
    exceed_lines: BTreeSet<u32>,

    s: String,
}

impl Debug for Typesetter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "pos: ({}, {})\n", self.line, self.col));
        try!(write!(f, "indent: \"{}\"\n", self.indent));
        try!(write!(f, "align stack: "));
        try!(Debug::fmt(&self.align_stack, f));
        try!(write!(f, "\nexceed lines: "));
        try!(Debug::fmt(&self.exceed_lines, f));
        Ok(())
    }
}

#[macro_export]
macro_rules! need_wrap {
    ($ts: expr, $($s: expr),+) => ({
        $ts.need_wrap(&[$($s),+])
    })
}

macro_rules! raw_insert {
    ($sf: expr, $s: expr) => ({
        $sf.s.push_str($s);

        $sf.col += $s.len();
        if $sf.col > MAX_WIDTH {
            $sf.exceed_lines.insert($sf.line);
        }
    })
}

impl Typesetter {
    pub fn new() -> Typesetter {
        Typesetter {
            line: 0,
            col: 0,
            indent: String::new(),
            align_stack: Vec::new(),
            exceed_lines: BTreeSet::new(),

            s: String::new(),
        }
    }

    pub fn result(self) -> (String, BTreeSet<u32>) {
        (self.s, self.exceed_lines)
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
    pub fn need_wrap(&mut self, list: &[&str]) -> bool {
        let len: usize = list.iter().map(|s| s.len()).sum();
        len > self.left() && len <= self.nl_left()
    }

    #[inline]
    pub fn wrap(&mut self) {
        self.nl();

        if self.should_align() {
            self.insert_align();
        } else {
            self.insert_indent();
            self.insert_wrap();
        }
    }

    #[inline]
    pub fn insert_mark_align(&mut self, s: &str) {
        self.insert(s);
        self.mark_align();
    }

    #[inline]
    pub fn insert_unmark_align(&mut self, s: &str) {
        self.insert(s);
        self.unmark_align();
    }

    #[inline]
    pub fn nl(&mut self) {
        self.s.push_str(NL);

        self.line += 1;
        self.col = 0;
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
    fn left(&self) -> usize {
        minus_nf!(MAX_WIDTH, self.col)
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
    fn wrap_insert(&mut self, s: &str) {
        self.wrap();
        self.insert(s);
    }

    #[inline]
    fn insert_wrap(&mut self) {
        self.raw_insert(WRAP_INDENT);
    }

    #[inline]
    fn insert_align(&mut self) {
        let blank = zstr::new_fill(' ', *self.align_stack.last().unwrap());
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
