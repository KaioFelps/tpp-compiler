# Implementação em Rust

Esse documento contém detalhes da implementação do compilador do TPP em Rust.
A abordagem adotada é a manual: REGEXes serão utilizados para identificar lexemas,
e uma série de `match`es resolverá o token.

As estruturas seguirão fortemente os exemplos, e a mesma interface adotada pelas
implementações em Python será utilizada na estrutura do Lexer em Rust.

## Pendências da primeira entrega
- [ ] flag `-h` ou `--help` deve mostrar uma mensagem listando todos os comandos e
    como usar o compilador.
- [ ] flag `-k` deve imprimir as chaves dos erros ao invés das mensagens.
- [ ] a chamada do compilador deve emitir uma lista dos tokens extraídos do código-fonte.

A flag `--lexer` não será implementada, visto que somente a implementação manual
será adotada, como dito anteriormente. As demais flags não serão implementadas nesta
entrega.
