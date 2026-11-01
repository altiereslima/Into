#!/usr/bin/env python3
"""Roda o jogo da cópia preparada, protegendo os dados do usuário.

Uso: python ferramentas/diferencial/rodar.py <pasta da cópia> [--tempo 90]

O jogo grava em Documents/My Games/Into The Breach mesmo rodando de outra pasta. Este script guarda essa pasta
antes, roda o jogo e depois devolve exatamente o que havia: restaura o que mudou e apaga o que apareceu.
"""
import argparse
import hashlib
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


def assinaturas(pasta):
    return {p.relative_to(pasta): hashlib.sha256(p.read_bytes()).hexdigest() for p in pasta.rglob('*') if p.is_file()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('copia', type=Path)
    ap.add_argument('--tempo', type=int, default=90)
    a = ap.parse_args()
    copia = a.copia.resolve()
    if not (copia / 'scripts' / 'diferencial_coleta.lua').is_file():
        raise SystemExit(f'{copia} não foi preparada com preparar.py')
    dados = Path(os.environ['USERPROFILE']) / 'Documents' / 'My Games' / 'Into The Breach'
    antes = assinaturas(dados) if dados.is_dir() else {}
    with tempfile.TemporaryDirectory() as tmp:
        guarda = Path(tmp) / 'dados'
        if dados.is_dir():
            shutil.copytree(dados, guarda)
        (copia / 'diff_original.txt').unlink(missing_ok=True)
        try:
            r = subprocess.run([str(copia / 'Breach.exe')], cwd=copia, timeout=a.tempo, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            print('jogo terminou com código', r.returncode)
        except subprocess.TimeoutExpired:
            print(f'tempo esgotado ({a.tempo} s): o jogo foi fechado')
        depois = assinaturas(dados) if dados.is_dir() else {}
        mudou = sorted(str(k) for k in antes if depois.get(k) != antes[k])
        novos = sorted(k for k in depois if k not in antes)
        print('dados do usuário alterados pelo jogo:', mudou, '| novos:', [str(k) for k in novos])
        for k in antes:
            if depois.get(k) != antes[k]:
                shutil.copy2(guarda / k, dados / k)
        for k in novos:
            (dados / k).unlink()
        print('dados restaurados:', assinaturas(dados) == antes)


if __name__ == '__main__':
    main()
