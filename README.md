# Compilador T++ Rust

Para o relatório da primeira parte do trabalho, dirija-se ao arquivo [Relatório 1.pdf](./Relatório%201.pdf).
(O código LaTeX do relatório está no diretório `./reports/`.) Para as notas de implementação tomadas durante
o desenvolvimento, veja o arquivo [rust-impl.md](./rust-impl.md).

> [!WARNING]
> Parte do código-fonte Rust pode estar misturado com o código-fonte Python.
> No geral, os módulos Python são prefixados com `tpp` (por exemplo, `tppparser/` ou `tpplexer/`),
> enquanto os do Rust, não (`parser/` ou `lexer`/`).

## Compilando
Para compilar o compilador TPP (Rust), utilize o comando `cargo build --release` e acesse o executável
em `./target/release/tppc <ARGUMENTOS>`.

Opcionalmente, você pode rodar o binário pelo cargo utilizando `cargo run --release -- <ARGUMENTOS>`.

## Uso
> [!WARNING]
> É necessário ter o programa `dot` instalado na sua máquina:
> ```sh
> winget install Graphviz.Graphviz # windows
> brew install graphviz # macos
> sudo apt update && sudo apt install graphviz # linux
> ```

Considere `tppc` o executável final (veja a seção [Compilando](#compilando)).

Utilize a _flag_ `--help` para ter detalhes de todas as opções do compilador. As principais estão listadas
adiante:
- `-s <lexer | parser>`: faz com que o compilador execute até o ponto de parada explicitado.
- `-k`: faz com que os erros sejam retornados em suas chaves ao invés de mensagens ricas.
- `-i`: faz com que os identificadores utilizem o subconjunto de identificadores válidos (isso remove
        a maioria dos caracteres utf-8, deixando os poucos que também são aceitos pelas implementações
        em Python).

### Exemplo
```sh
alias tppc=./target/release/tppc
tppc -s lexer ./teste.tpp # executa o compilador até a fase léxica
tppc -s lexer -k ./teste.tpp # executa o compilador até a fase léxica e imprime as chaves dos erros
tppc ./teste.tpp # executa o fluxo completo do compilador normalmente
```

## Testes
Para rodar os testes, basta executar os comandos assim como já era feito no template: `pytest -v --lexer rust`.
Os arquivos de _benchmark_ e demais testes também foram adaptados e podem ser usados normalmente,
com a opção _lexer_ `rust` onde aplicável.

## Implementações em Python
Para utilizar as implementações em Python,
- instale o [uv](https://docs.astral.sh/uv/);
- instale as bibliotecas necessárias (`uv sync`);
- execute o script principal (`uv run src/tpp_compiler.py`).

Para as documentações sobre as implementações em Python, refira-se ao [read-me antigo](./old_README.md).
