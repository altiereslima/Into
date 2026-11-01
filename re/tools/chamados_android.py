#!/usr/bin/env python3
"""Lista, no libmain.so (Android 1.2.92), os chamados de cada função cujo nome contém um dos padrões.

Uso: python -I re/tools/chamados_android.py <libmain.so> "Board::Load(" "BoardPlayer::SelectMap" ...
Precisa de c++filt no PATH (WinLibs).

Serve para dar nome a uma rotina do PC sem nome casado: a função que a chama costuma ter nome, e no Android os chamados
dela aparecem com símbolo, na mesma ordem (salvo o que um compilador embutiu e o outro não). Registre o resultado em
re/evidencias/v1.2.93/nomes_manual.csv, com a prova.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from match_android_symbols import load_arm  # noqa: E402


def main():
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    funcs, _ = load_arm(sys.argv[1])
    for pat in sys.argv[2:]:
        for f in sorted(funcs.values(), key=lambda f: f['start']):
            if pat not in f['name']:
                continue
            print(f"== {f['name']}  @{f['start']:#x} ({f['size']} bytes)")
            for c in f['calls']:
                g = funcs.get(c)
                print('   ', g['name'] if g else hex(c))
            if f['strings']:
                print('    textos:', f['strings'])


if __name__ == '__main__':
    main()
