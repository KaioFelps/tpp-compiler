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