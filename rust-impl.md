# Implementação em Rust

Esse documento contém detalhes da implementação do compilador do TPP em Rust.
A abordagem adotada é a manual: REGEXes serão utilizados para identificar lexemas,
e uma série de `match`es resolverá o token. Além do mais, uma tabela de símbolos será
consultada para decidir quando um identificador é, na verdade, uma palavra reservada.

As estruturas seguirão fortemente os exemplos, e a mesma interface adotada pelas
implementações em Python será utilizada na estrutura do Lexer em Rust.

## Pendências da primeira entrega
- [ ] _flag_ `-h` ou `--help` deve mostrar uma mensagem listando todos os comandos e
    como usar o compilador.
- [ ] _flag_ `-k` deve imprimir as chaves dos erros ao invés das mensagens.
- [ ] a chamada do compilador deve emitir uma lista dos tokens extraídos do código-fonte.

A _flag_ `--lexer` não será implementada, visto que somente a implementação manual
será adotada, como dito anteriormente. As demais flags não serão implementadas nesta
entrega.

A _crate_ `clap` é uma popular biblioteca no ecossistema Rust especializada em
interpretar comandos e flags de aplicativos de linha de comando. Por isso, foi utilizada
para montar a interface do compilador, isto é, lidar com as _flags_ e outros argumentos.

## Erros
Os arquivos `*.properties` no formato _ini_ são utilizados para descrever erros e
categorizá-los de acordo com o domínio ao qual pertencem. Eles também foram utilizados
na implementação em Rust do compilador, e são interpretados utilizando a _crate_
`configparser`.

No entanto, diferente da implementação em Python, nesta versão, mantemos todas as
seções e os erros em memória ao realizar a leitura desses arquivos apenas uma vez
durante a inicialização do compilador. Essa decisão não tem nenhum impacto na performance
do compilador na prática, visto que diante do primeiro erro, o programa se encerra.

A biblioteca _anyhow_ foi utilizada para facilitar a propagação de erros.

O arquivo `LexerErrorMessages.properties` foi copiado para `__LexerErrorMessages.properties`,
e todas as referências a ele nas implementações em Python agora apontam para esta cópia.
A versão original contém mensagens de erros mais ricas, isto é, com mais informações sobre
o contexto do problema, incluindo o caminhod o arquivo, o provável lexema, a linha e a coluna
em que estão localizados no arquivo do código-fonte.

## Caracteres ou Bytes
Em C/C++, um caractere é sinônimo de byte. De fato, um caractere ASCII ocupa exatamente
1 byte, e para muitos textos, isso é o suficiente para indexar corretamente cada caractere.

Como a linguagem TPP tem, em seu código fonte, caracteres unicode por padrão — visto que
as próprias palavras chaves contêm acentos —, indexar os bytes de uma string Rust não é
correto.

Observe que, se símbolos ainda mais complexos fossem incluídos na linguagem, como "y̆", nem
mesmo os `char`s do Rust — que são, na verdade, 4 bytes e representam um valor unicode —
seriam suficiente. Como não é o caso, os `char`s ainda servem.

Contudo, embora a biblioteca padrão do Rust entregue o método `String::chars` para obter um
iterador sobre cada caractere de uma string, esse iterador permite performar um _lookahead_
de, no máximo, 1 caractere. Isso não é o bastante para agrupar, manualmente, lexemas maiores.

Diante disso, será utilizada a _crate_ _itertools_, uma biblioteca popular e padrão da
comunidade, para iterar sobre os caracteres (`char`) da string do código-fonte de forma mais
robusta, isto é, sem a necessidade de criar-se iteradores e percorrer a string $n$ vezes
todas as vezes que se quiser acessar o `char` na posição $n$, ou de iterar apenas uma vez
sobre os caracteres (usando o iterador `Chars` do Rust) sob a limitação de um _lookahead_
mínimo.

A funcionalidade `multipeek` do _itertools_ será utilizada para performar a iteração sobre
a string com um _lookahead_ dinâmico, isto é, limitado apenas pelo tamanho da própria string.
Isso se difere da implementação manual em Python provida como exemplo, que itera mantendo um
cursor relativo à classe do lexer, e que implementa um método `_peek` que visualiza outro
caractere sem avançar o cursor. Também substitui uma tentativa realizada na implementação em
Rust que partia do mesmo princípio, porém que implementava um método `peek(offset)` que
buscava o caractere na posição $string_{\mathit{cursor} + \mathit{offset}}$.

## Identificadores
A implementação do lexer em Rust considera como caractere válido de um
identificador — isto é, nome de variável, de função, etc — qualquer letra seguida de uma
quantidade variável de letras, números ou _underscores_.

Uma letra, por padrão, não é apenas o alfabeto simples, mas sim agrupamento de grafemas, o que
inclui caracteres de outros idiomas, como kanji ou letras com diacríticos, como 'ç'. As
implementações em Python, contudo, consideram apenas um subconjunto do que consideramos uma letra.

Assim, a fim de reaproveitar os testes já escritos e fazer uma comparação simples com essas
outras implementações, foi introduzida as _flags_ `-i` ou `--id-subset`, que forçam o _lexer_
Rust a reconhecer somente as mesmas letras que as demais implementações.

Essa opção é desabilitada por padrão, porém será sempre habilitada em todos os testes que comparam
as saídas de erros e tokens entre as implementações a fim de estabelecer compatibilidade.

## Números com Sinais
Números podem ser definidos com ou sem sinal, isto é, -1, 1, +1 são definições válidas de um número.
Num primeiro momento, o lexer agruparia todo sinal seguido de números como um número decimal ou
inteiro. Isso, no entanto, pode causar conflitos quando se estiver analisando uma expressão (e não
uma atribuição/sentença). Por exemplo, em `a+2`, é esperado que se obtenha os _tokens_
$(\mathit{ID},\mathit{MAIS},\mathit{NUM_INTEIRO})$. No entanto, com esse agrupamento, seria obtido
somente $(\mathit{ID},\mathit{NUM_INTEIRO})$.

Assim, alterou-se a implementação para considerar todo `+` e `-` exclusivamente como os seus próprios
_tokens_, isto é, não mais compondo inteiros. Dessa forma, será responsabilidade da análise sintática,
a ser realizada posteriormente, ditar se o símbolo é um sinal do número ou um operador.

## Testes
Utilizando uma IA, foram realizadas modificações nos arquivos de testes para que considerassem
o compilador — que, até o momento, somente possui o serviço de análise léxica — em Rust e o utilizassem
corretamente para performar as asserções e os comparativos (_benchmarks_).

As alterações realizadas foram, em suma, a adição da opção de _lexer_ `rust` e de condicionais para
compiliar o programa em modo _release_ e executar o programa binário ao invés do script Python
`tpp_compiler.py`.

Os comandos que executam o programa em Rust contém a _flag_ `-i`, como discutido na seção [Identificadores].
Em alguns casos, onde se utiliza a função `get_lexer` para executar o analisador diretamente, houve de ser
criado um _mock_ (`RustMockLexer`) para interfacear com o programa em Rust, visto que não é possível realizar
chamadas para o analisador léxico diretamente do script Python sem uso de FFI — o que seria um trabalho em
demásia e que não agregaria nada de relevante para o trabalho.

[Identificadores]: #identificadores
