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
    value: &'a str,
    line_number: usize,
    col_number: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, value: &'a str, line_number: usize, col_number: usize) -> Self {
        Self {
            kind,
            value,
            line_number,
            col_number,
        }
    }
}
