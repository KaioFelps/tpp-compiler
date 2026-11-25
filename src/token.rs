use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    AbreChave,
    FechaChave,
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
    FimDeSentenca,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct OwnedToken {
    pub kind: TokenKind,
    pub value: String,
    pub value_begin_pos: usize,
    pub line_number: usize,
    pub col_number: usize,
}

pub trait IntoOnwedToken {
    fn make_owned(&self) -> OwnedToken;
}

impl IntoOnwedToken for Token<'_> {
    fn make_owned(&self) -> OwnedToken {
        OwnedToken {
            kind: self.kind.clone(),
            value: self.value.to_string(),
            value_begin_pos: self.value_begin_pos,
            line_number: self.line_number,
            col_number: self.col_number,
        }
    }
}

impl IntoOnwedToken for &Token<'_> {
    fn make_owned(&self) -> OwnedToken {
        OwnedToken {
            kind: self.kind.clone(),
            value: self.value.to_string(),
            value_begin_pos: self.value_begin_pos,
            line_number: self.line_number,
            col_number: self.col_number,
        }
    }
}

impl IntoOnwedToken for OwnedToken {
    fn make_owned(&self) -> OwnedToken {
        self.clone()
    }
}

impl IntoOnwedToken for &OwnedToken {
    fn make_owned(&self) -> OwnedToken {
        (*self).clone()
    }
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

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn value(&self) -> &str {
        self.value
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn value_begin_pos(&self) -> usize {
        self.value_begin_pos
    }

    pub fn col_number(&self) -> usize {
        self.col_number
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
            Self::AbreChave => "ABRE_CHAVE",
            Self::FechaChave => "FECHA_CHAVE",
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
            Self::FimDeSentenca => "END_OF_STATEMENT",
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

pub(super) trait IntoKind {
    fn into_kind(&self) -> Option<TokenKind>;
}

impl<'t> IntoKind for Option<Token<'t>> {
    fn into_kind(&self) -> Option<TokenKind> {
        self.as_ref().map(|token| token.kind.clone())
    }
}

impl<'t> IntoKind for Option<&Token<'t>> {
    fn into_kind(&self) -> Option<TokenKind> {
        self.map(|token| token.kind.clone())
    }
}

pub(super) trait CanBeOfKind {
    fn is_of_kind(&self, kind: TokenKind) -> bool;
    fn is_any_of_kinds(&self, kinds: &[TokenKind]) -> bool;
}

impl<T: IntoKind> CanBeOfKind for T {
    fn is_of_kind(&self, kind: TokenKind) -> bool {
        self.into_kind().is_some_and(|actual_kind| actual_kind == kind)
    }

    fn is_any_of_kinds(&self, kinds: &[TokenKind]) -> bool {
        self.into_kind().is_some_and(|actual_kind| kinds.contains(&actual_kind))
    }
}

impl<'t> CanBeOfKind for Token<'t> {
    fn is_of_kind(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    fn is_any_of_kinds(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.kind)
    }
}
