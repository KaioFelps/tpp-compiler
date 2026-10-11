use std::fmt::{Display, write};

#[derive(Debug, Clone)]
pub enum TokenKind {
    Ident,
    Mais,
    Menos,
    Vezes,
    Divide,
    DoisPontos,
    Virgula,
    Menor,
    Maior,
    Igual,
    Diferente,
    MenorIgual,
    MaiorIgual,
    E,
    Ou,
    Nao,
    AbreParentese,
    FechaParentese,
    AbreColchete,
    FechaColchete,
    Se,
    Entao,
    Senao,
    Fim,
    Repita,
    Ate,
    Atribuicao,
    Leia,
    Escreva,
    Retorna,
    Inteiro,
    Flutuante,
    NumInteiro,
    NumPontoFlutuante,
    NumNotacaoCientific,
}

#[derive(Debug)]
pub struct Token<'a> {
    kind: TokenKind,
    /// A window of the value from the actual source code string.
    value: &'a str,
    /// The index (in the source code string) of the begin of `value`.
    value_begin_pos: usize,
    /// The line (per line breaks) where the value is on.
    line_number: usize,
    /// The column (relative to the line in the source code file) where the value begins.
    col_number: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, value: &'a str, value_begin_pos: usize, line_number: usize, col_number: usize) -> Self {
        Self {
            kind,
            value,
            value_begin_pos,
            line_number,
            col_number,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{:?}("{}") (linha {}, coluna {})"#,
            self.kind, self.value, self.line_number, self.col_number
        )
    }
}
