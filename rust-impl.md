# Implementação em Rust

Esse documento contém detalhes da implementação do compilador do TPP em Rust.
A abordagem adotada é a manual: REGEXes serão utilizados para identificar lexemas,
e uma série de `match`es resolverá o token. Além do mais, uma tabela de símbolos será
consultada para decidir quando um identificador é, na verdade, uma palavra reservada.

As estruturas seguirão fortemente os exemplos, e a mesma interface adotada pelas
implementações em Python será utilizada na estrutura do Lexer em Rust.

## Pendências da primeira entrega
- [x] _flag_ `-h` ou `--help` deve mostrar uma mensagem listando todos os comandos e
    como usar o compilador.
- [x] _flag_ `-k` deve imprimir as chaves dos erros ao invés das mensagens.
- [x] a chamada do compilador deve emitir uma lista dos tokens extraídos do código-fonte.

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
o contexto do problema, incluindo o caminho do arquivo, o provável lexema, a linha e a coluna
em que estão localizados no arquivo do código-fonte.

## Caracteres ou Bytes
Em C/C++, um caractere é sinônimo de byte. De fato, um caractere ASCII ocupa exatamente
1 byte, e para muitos textos, isso é o suficiente para indexar corretamente cada caractere.

Como a linguagem TPP tem, em seu código-fonte, caracteres unicode por padrão — visto que
as próprias palavras-chave contêm acentos —, indexar os bytes de uma string Rust não é
correto.

Observe que, se símbolos ainda mais complexos fossem incluídos na linguagem, como "y̆", nem
mesmo os `char`s do Rust — que são, na verdade, 4 bytes e representam um valor unicode —
seriam suficientes. Como não é o caso, os `char`s ainda servem.

Contudo, embora a biblioteca padrão do Rust entregue o método `String::chars` para obter um
iterador sobre cada caractere de uma string, esse iterador permite performar um _lookahead_
de, no máximo, 1 caractere. Isso não é o bastante para agrupar, manualmente, lexemas maiores.

Diante disso, será utilizada a _crate_ _itertools_, uma biblioteca popular e padrão da
comunidade, para iterar sobre os caracteres (`char`) da string do código-fonte de forma mais
robusta, isto é, sem a necessidade de se criar iteradores e percorrer a string $n$ vezes
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
identificador — isto é, nome de variável, de função, etc. — qualquer letra seguida de uma
quantidade variável de letras, números ou _underscores_.

Uma letra, por padrão, não é apenas o alfabeto simples, mas sim um agrupamento de grafemas, o que
inclui caracteres de outros idiomas, como kanji ou letras com diacríticos, como 'ç'. As
implementações em Python, contudo, consideram apenas um subconjunto do que consideramos uma letra.

Assim, a fim de reaproveitar os testes já escritos e fazer uma comparação simples com essas
outras implementações, foram introduzidas as _flags_ `-i` ou `--id-subset`, que forçam o _lexer_
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

Os comandos que executam o programa em Rust contêm a _flag_ `-i`, como discutido na seção [Identificadores].
Em alguns casos, onde se utiliza a função `get_lexer` para executar o analisador diretamente, houve de ser
criado um _mock_ (`RustMockLexer`) para interfacear com o programa em Rust, visto que não é possível realizar
chamadas para o analisador léxico diretamente do script Python sem uso de FFI — o que seria um trabalho em
demasia e que não agregaria nada de relevante para o trabalho.

[Identificadores]: #identificadores

## Benchmarks
Após configurar o arquivo de _benchmarks_, o Rust apresentava execuções com resultados com altas
variações: ora inferior às implementações em Python, ora superior:

### Execução 1 (sem warm-up do _lexer_ Rust)
| Estratégia | Tokens | Tempo Médio (s) | Tempo Mín (s) | Vazão (tokens/s) | Relativo (vs PLY) |
| --- | --- | --- | --- | --- | --- |
| ply | 73.000 | 0.0949s | 0.0896s | 769.214 tok/s | 1.00x (baseline) |
| mandfa | 73.000 | 0.1178s | 0.1097s | 619.927 tok/s | 0.81x |
| symtable | 73.000 | 0.1291s | 0.1198s | 565.520 tok/s | 0.74x |
| symtableman | 73.000 | 0.1388s | 0.1251s | 526.034 tok/s | 0.68x |
| automatalib | 73.000 | 0.1126s | 0.1091s | 648.559 tok/s | 0.84x |
| automatalibman | 73.000 | 0.1118s | 0.1082s | 652.681 tok/s | 0.85x |
| rust | 73.000 | 0.1915s | 0.0374s | 381.260 tok/s | 0.50x |

### Execução 2 (sem warm-up do _lexer_ Rust)
| Estratégia | Tokens | Tempo Médio (s) | Tempo Mín (s) | Vazão (tokens/s) | Relativo (vs PLY) |
| --- | --- | --- | --- | --- | --- |
| ply | 73.000 | 0.1046s | 0.0881s | 698.003 tok/s | 1.00x (baseline) |
| mandfa | 73.000 | 0.1125s | 0.1074s | 649.014 tok/s | 0.93x |
| symtable | 73.000 | 0.1279s | 0.1241s | 570.926 tok/s | 0.82x |
| symtableman | 73.000 | 0.1332s | 0.1321s | 548.186 tok/s | 0.79x |
| automatalib | 73.000 | 0.1130s | 0.1093s | 645.945 tok/s | 0.93x |
| automatalibman | 73.000 | 0.1178s | 0.1168s | 619.478 tok/s | 0.89x |
| rust | 73.000 | 0.0664s | 0.0384s | 1.099.162 tok/s | 1.57x |

### Execução 3 (sem warm-up do _lexer_ Rust)
| Estratégia | Tokens | Tempo Médio (s) | Tempo Mín (s) | Vazão (tokens/s) | Relativo (vs PLY) |
| --- | --- | --- | --- | --- | --- |
| ply | 73.000 | 0.0930s | 0.0914s | 784.900 tok/s | 1.00x (baseline) |
| mandfa | 73.000 | 0.1155s | 0.1132s | 631.982 tok/s | 0.81x |
| symtable | 73.000 | 0.1250s | 0.1214s | 584.010 tok/s | 0.74x |
| symtableman | 73.000 | 0.1293s | 0.1272s | 564.473 tok/s | 0.72x |
| automatalib | 73.000 | 0.1148s | 0.1100s | 636.017 tok/s | 0.81x |
| automatalibman | 73.000 | 0.1153s | 0.1111s | 632.955 tok/s | 0.81x |
| rust | 73.000 | 0.0893s | 0.0368s | 817.339 tok/s | 1.04x 

Ao adicionar uma execução de warm-up — que neste caso, serve apenas para carregar o binário executável
na memória RAM, não para performar algumas compilações (JIT) como nas versões com Python —, os resultados
melhoraram significativamente, mostrando uma eficiência quase 3 vezes maior em relação à implementação
base (PLY).

### Execução 1 (com warm-up do _lexer_ Rust)
| Estratégia | Tokens | Tempo Médio (s) | Tempo Mín (s) | Vazão (tokens/s) | Relativo (vs PLY) |
| --- | --- | --- | --- | --- | --- |
| ply | 73.000 | 0.0888s | 0.0871s | 821.789 tok/s | 1.00x (baseline) |
| mandfa | 73.000 | 0.1155s | 0.1127s | 632.132 tok/s | 0.77x |
| symtable | 73.000 | 0.1235s | 0.1185s | 591.183 tok/s | 0.72x |
| symtableman | 73.000 | 0.1273s | 0.1240s | 573.543 tok/s | 0.70x |
| automatalib | 73.000 | 0.1121s | 0.1088s | 651.256 tok/s | 0.79x |
| automatalibman | 73.000 | 0.1145s | 0.1117s | 637.771 tok/s | 0.78x |
| rust | 73.000 | 0.0402s | 0.0373s | 1.818.057 tok/s | 2.21x |

### Execução 2 (com warm-up do _lexer_ Rust)
| Estratégia | Tokens | Tempo Médio (s) | Tempo Mín (s) | Vazão (tokens/s) | Relativo (vs PLY) |
| --- | --- | --- | --- | --- | --- |
| ply | 73.000 | 0.1118s | 0.0906s | 652.892 tok/s | 1.00x (baseline) |
| mandfa | 73.000 | 0.1200s | 0.1185s | 608.399 tok/s | 0.93x |
| symtable | 73.000 | 0.1323s | 0.1309s | 551.981 tok/s | 0.85x |
| symtableman | 73.000 | 0.1271s | 0.1226s | 574.335 tok/s | 0.88x |
| automatalib | 73.000 | 0.1122s | 0.1090s | 650.529 tok/s | 1.00x |
| automatalibman | 73.000 | 0.1267s | 0.1193s | 576.277 tok/s | 0.88x |
| rust | 73.000 | 0.0401s | 0.0393s | 1.819.951 tok/s | 2.79x |

### Execução 3 (com warm-up do _lexer_ Rust)
| Estratégia | Tokens | Tempo Médio (s) | Tempo Mín (s) | Vazão (tokens/s) | Relativo (vs PLY) |
| --- | --- | --- | --- | --- | --- |
| ply | 73.000 | 0.0929s | 0.0880s | 785.667 tok/s | 1.00x (baseline) |
| mandfa | 73.000 | 0.1127s | 0.1067s | 647.626 tok/s | 0.82x |
| symtable | 73.000 | 0.1265s | 0.1201s | 577.093 tok/s | 0.73x |
| symtableman | 73.000 | 0.1271s | 0.1250s | 574.222 tok/s | 0.73x |
| automatalib | 73.000 | 0.1132s | 0.1096s | 644.949 tok/s | 0.82x |
| automatalibman | 73.000 | 0.1121s | 0.1087s | 650.919 tok/s | 0.83x |
| rust | 73.000 | 0.0366s | 0.0351s | 1.996.396 tok/s | 2.54x |
