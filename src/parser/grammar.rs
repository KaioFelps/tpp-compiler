use itertools::Either;

use crate::{errors::TppCompilerError, parser::cst::CstNode};

mod array;
mod block;
mod declaration;
mod expression;
mod factor;
mod function;
mod number;
mod operator;
mod program;
mod tuple;
mod r#type;
mod var;

/// The return type of a final grammar parsing method.
///
/// The functions that return this type are final, i.e., they must be called
/// only when it's assured that they **are** the ones for parsing the tokins sequence.
///
/// For internal methods, see [`GrammarInnerParseFailable`].
pub type GrammarFinalParseResult<'t> = Result<CstNode<'t>, TppCompilerError>;

/// The return type of a final grammar parsing method that performs a lookahead peek
/// to check whether it can actually generate the resulting [`CstNode`] and early-returns
/// `None` if not.
pub type GrammarFinalParseResultWithLookahead<'t> = Result<Option<CstNode<'t>>, TppCompilerError>;

/// The return type of inner grammar parsing methods.
/// This monad allows chaining multiple inner parsing methods until one of them matches.
///
/// Every method returning this type **must** ensure it can actually parse the token sequence.
/// Hence, it must firstly checks that the following token (via peeking) is an element of its
/// *First* (grammar) set, and return a `Right(token_from_parameter)` otherwise.
pub type GrammarInnerParseFailable<'t> = Result<Either<CstNode<'t>, CstNode<'t>>, TppCompilerError>;

pub(super) trait InnerParseChain<'t> {
    fn on_right_try(
        self,
        f: impl FnOnce(CstNode<'t>) -> GrammarInnerParseFailable<'t>,
    ) -> GrammarInnerParseFailable<'t>;
}

impl<'t> InnerParseChain<'t> for GrammarInnerParseFailable<'t> {
    fn on_right_try(
        self,
        f: impl FnOnce(CstNode<'t>) -> GrammarInnerParseFailable<'t>,
    ) -> GrammarInnerParseFailable<'t> {
        match self? {
            Either::Left(node) => Ok(Either::Left(node)),
            Either::Right(node) => f(node),
        }
    }
}

#[derive(Debug)]
pub enum NonTerminal {
    Program,
    Declaration,
    VarDeclaration,
    VarInitialization,
    Attribution,
    Var,
    Type,
    FuncDeclaration,
    ReturnType,
    FuncHeader,
    Param,
    Block,
    Action,
    If,
    Repeat,
    TargetList,
    ExprList,
    Read,
    Write,
    Return,
    Expr,
    SimpleExpr,
    AdditionExpr,
    ProductExpr,
    UnaryExpr,
    RelationalOperator,
    LogicalOperator,
    SumOperator,
    NotOperator,
    ProductOperator,
    Factor,
    Tuple,
    VecInitialization,
    Number,
    FuncCall,
}
