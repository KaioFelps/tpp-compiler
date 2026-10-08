# Implementação em Rust

Esse documento contém detalhes da implementação do compilador do TPP em Rust.
A abordagem adotada é a manual: REGEXes serão utilizados para identificar lexemas,
e uma série de `match`es resolverá o token.

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