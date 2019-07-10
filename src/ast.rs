pub use syntax::ast::*;
pub use syntax::parse::lexer::comments::*;
pub use syntax::parse::token::{self, Token, TokenKind};
pub use syntax::ptr::*;
pub use syntax::tokenstream::*;
pub use syntax_pos::*;
pub use syntax_pos::symbol::*;

pub type TokenLit = token::Lit;
