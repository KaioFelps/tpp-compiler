import pytest
import subprocess
import shlex
import sys
import os
import fnmatch


def pytest_generate_tests(metafunc):
    """Gera dinamicamente os casos de teste com base na opção --parser."""
    parser_opt = metafunc.config.getoption("parser")

    if parser_opt == "all":
        parsers = ["yacc", "rust"]
    else:
        parsers = [parser_opt]

    # Testes para se passar um arquivo em branco, um arquivo com outra extensão e um arquivo .tpp que não existe.
    cases = []
    for parser in parsers:
        cases.append((parser, "-k", ""))
        cases.append((parser, "-k", "teste.c"))
        cases.append((parser, "-k", "notexist.tpp"))

    # Arquivos de teste .tpp
    files = sorted(fnmatch.filter(os.listdir('tests/syn-tests/'), '*.tpp'))
    for file in files:
        for parser in parsers:
            cases.append((parser, "-k", file))

    metafunc.parametrize("parser_type, key_option, input_file", cases)


def test_execute(parser_type, key_option, input_file):
    if input_file != "":
        path_file = "tests/syn-tests/" + input_file
    else:
        path_file = ""

    if parser_type == "rust":
        bin_path = "./target/release/tppc"
        if sys.platform == "win32":
            bin_path += ".exe"

        cmd = [bin_path, "-i", "-s", "parser", "--output-cst", key_option]
        if path_file:
            cmd.append(path_file)

        # Redireciona stderr para stdout para capturar erros léxicos/sintáticos impressos no stderr
        process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    else:
        # Executa o comando de compilação. O -t gera a árvore sintática (e os arquivos
        # .ast.dot, .unique.ast.dot, .unique.ast.png e ascii_tree.txt ao lado do .tpp).
        cmd = f"python src/tpp_compiler.py --parser {parser_type} {key_option} -t {path_file}"
        process = subprocess.Popen(shlex.split(cmd), stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    stdout, stderr = process.communicate()

    output_path = "tests/syn-tests/" + input_file + ".syn.out"
    with open(output_path, "r", encoding="utf-8") as output_file:
        expected_output = output_file.read()

    generated_output = stdout.decode("utf-8").strip()

    print("Generated output:")
    print(generated_output)
    print("Expected output:")
    print(expected_output)

    assert generated_output == expected_output.strip()


if __name__ == "__main__":
    sys.exit(pytest.main(sys.argv[1:] + [__file__]))
