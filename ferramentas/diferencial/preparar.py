#!/usr/bin/env python3
"""Monta uma cópia da instalação com a coleta do teste diferencial. O original não é alterado.

Uso: python ferramentas/diferencial/preparar.py <pasta do jogo> <pasta da cópia> [--modo boot]

A cópia NUNCA deve ficar dentro do repositório. `resources/` entra por hard link, porque são arquivos grandes
que o jogo só lê; o resto é copiado. Só `scripts/scripts.lua` da cópia muda: ganha a coleta no fim da lista.
"""
import argparse
import os
import shutil
from pathlib import Path

AQUI = Path(__file__).resolve().parent
REPO = AQUI.parents[1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('jogo', type=Path)
    ap.add_argument('copia', type=Path)
    ap.add_argument('--modo', default='boot')
    a = ap.parse_args()
    jogo, copia = a.jogo.resolve(), a.copia.resolve()
    if not (jogo / 'Breach.exe').is_file():
        raise SystemExit(f'{jogo} não tem Breach.exe')
    if copia == jogo or jogo in copia.parents or REPO in copia.parents:
        raise SystemExit('a cópia precisa ficar fora do jogo e fora do repositório')

    copia.mkdir(parents=True, exist_ok=True)
    for item in jogo.iterdir():
        dest = copia / item.name
        if item.name == 'resources':
            dest.mkdir(exist_ok=True)
            for f in item.iterdir():
                if not (dest / f.name).exists():
                    os.link(f, dest / f.name)
        elif item.is_dir():
            shutil.copytree(item, dest, dirs_exist_ok=True)
        else:
            shutil.copy2(item, dest)

    coleta = (AQUI / 'coleta.lua').read_text(encoding='utf-8')
    cabecalho = f'DIF_SAIDA = "diff_original.txt"\nDIF_MODO = "{a.modo}"\n'
    (copia / 'scripts' / 'diferencial_coleta.lua').write_text(cabecalho + coleta, encoding='utf-8', newline='\n')
    lista = (jogo / 'scripts' / 'scripts.lua').read_bytes().decode('utf-8', 'replace')
    gancho = ('\nlocal dif_original = GetScripts\n'
              'function GetScripts()\n'
              '\tlocal t = dif_original()\n'
              '\tt[#t + 1] = "scripts/diferencial_coleta.lua"\n'
              '\treturn t\n'
              'end\n')
    (copia / 'scripts' / 'scripts.lua').write_text(lista + gancho, encoding='utf-8', newline='\n')
    print(f'cópia pronta em {copia}, modo {a.modo}. Rode Breach.exe a partir dessa pasta; a saída é diff_original.txt.')


if __name__ == '__main__':
    main()
