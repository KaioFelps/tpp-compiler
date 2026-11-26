inteiro: x

x := 10

{* não pode ter esse tipo de sentença/declaração no escopo global *}
se x > 0 então
	escreva(x)
senão
	escreva(x + 1)
fim

inteiro principal()
	se x > 10 então
		escreva(x)
	senão
		escreva(x + 1)
fim
