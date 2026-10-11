import subprocess
import sys
from pathlib import Path
import pytest

# Adiciona a raiz do projeto ao sys.path
sys.path.insert(0, str(Path(__file__).parent.parent))


def pytest_addoption(parser):
    parser.addoption(
        "--lexer",
        action="store",
        default="ply",
        choices=["all", "ply", "mandfa", "symtable", "symtableman", "automatalib", "automatalibman", "rust"],
        help="Especifica qual lexer testar: ply, mandfa, symtable, symtableman, automatalib, automatalibman, rust ou all (padrão: ply)",
    )

def pytest_collection_modifyitems(config, items):
    lexer = config.getoption("--lexer")
    if not lexer or lexer == "all":
        return

    selected = []
    deselected = []

    for item in items:
        # Verifica se o teste possui o parâmetro 'lexer_type'
        lexer_type = getattr(item, "callspec", None) and item.callspec.params.get("lexer_type")
        if lexer_type is not None:
            if lexer_type == lexer:
                selected.append(item)
            else:
                deselected.append(item)
        else:
            selected.append(item)

    if deselected:
        config.hook.pytest_deselected(items=deselected)
        items[:] = selected

        import subprocess

@pytest.fixture(scope="session", autouse=True)
def build_rust_binary(request):
    """Compila o projeto Rust em release uma única vez antes da sessão de testes."""
    lexer_opt = request.config.getoption("lexer")
    if lexer_opt == "all" or lexer_opt == "rust":
        print("\n[+] Compilando o binário Rust (release)...")
        subprocess.run(["cargo", "build", "--release", "-q"], check=True)