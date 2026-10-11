use std::fmt::Display;

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
    NumNotacaoCientifica,
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

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stringified = match self {
            Self::Ident => "ID",
            Self::Mais => "MAIS",
            Self::Menos => "MENOS",
            Self::Vezes => "VEZES",
            Self::Divide => "DIVIDE",
            Self::DoisPontos => "DOIS_PONTOS",
            Self::Virgula => "VIRGULA",
            Self::Menor => "MENOR",
            Self::Maior => "MAIOR",
            Self::Igual => "IGUAL",
            Self::Diferente => "DIFERENTE",
            Self::MenorIgual => "MENOR_IGUAL",
            Self::MaiorIgual => "MAIOR_IGUAL",
            Self::E => "E",
            Self::Ou => "OU",
            Self::Nao => "NAO",
            Self::AbreParentese => "ABRE_PARENTESE",
            Self::FechaParentese => "FECHA_PARENTESE",
            Self::AbreColchete => "ABRE_COLCHETE",
            Self::FechaColchete => "FECHA_COLCHETE",
            Self::Se => "SE",
            Self::Entao => "ENTAO",
            Self::Senao => "SENAO",
            Self::Fim => "FIM",
            Self::Repita => "REPITA",
            Self::Ate => "ATE",
            Self::Atribuicao => "ATRIBUICAO",
            Self::Leia => "LEIA",
            Self::Escreva => "ESCREVA",
            Self::Retorna => "RETORNA",
            Self::Inteiro => "INTEIRO",
            Self::Flutuante => "FLUTUANTE",
            Self::NumInteiro => "NUM_INTEIRO",
            Self::NumPontoFlutuante => "NUM_PONTO_FLUTUANTE",
            Self::NumNotacaoCientifica => "NUM_NOTACAO_CIENTIFICA",
        };

        write!(f, "{stringified}")
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{}("{}") (linha {}, coluna {})"#,
            self.kind, self.value, self.line_number, self.col_number
        )
    }
}
