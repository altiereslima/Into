#!/usr/bin/env python3
"""Uma rodada inteira do teste diferencial: prepara a cópia, roda o jogo protegendo os dados do usuário e, quando ele
fecha, compara com o motor Rust e grava o relatório.

Uso: python ferramentas/diferencial/rodada.py <pasta do jogo> <pasta da cópia> [--modo missao] [--tempo 900]
     [--relatorio docs/relatorios/diferencial_1293.txt]

O jogo não tem comando nem console que abra uma missão: no modo missão, alguém precisa passar pelo menu (sequência em
LEIA-ME.md). Com --modo boot não há cliques.
"""
import argparse
import subprocess
import sys
from pathlib import Path

AQUI = Path(__file__).resolve().parent
REPO = AQUI.parents[1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('jogo', type=Path)
    ap.add_argument('copia', type=Path)
    ap.add_argument('--modo', default='missao')
    ap.add_argument('--tempo', type=int, default=900)
    ap.add_argument('--relatorio', type=Path, default=REPO / 'docs' / 'relatorios' / 'diferencial_1293.txt')
    a = ap.parse_args()
    py = sys.executable
    subprocess.run([py, str(AQUI / 'preparar.py'), str(a.jogo), str(a.copia), '--modo', a.modo], check=True)
    if a.modo == 'missao':
        print('jogo abrindo: tecla na tela de Turret_Base, Novo jogo, Iniciar, Iniciar, clique na ilha', flush=True)
    subprocess.run([py, str(AQUI / 'rodar.py'), str(a.copia), '--tempo', str(a.tempo)], check=True)
    saida = a.copia / 'diff_original.txt'
    ultima = saida.read_bytes().decode('utf-8', 'replace').rstrip().splitlines()[-1:] if saida.exists() else []
    if not ultima or not ultima[0].startswith('fim|' + ('missao' if a.modo == 'missao' else 'menu')):
        print('a coleta não chegou ao fim; última linha:', ultima)
    r = subprocess.run(['cargo', 'run', '-q', '-p', 'itb-lua', '--bin', 'itb-diff', '--', str(a.jogo), str(saida)],
                       cwd=REPO, capture_output=True, text=True, encoding='utf-8')
    # o que os scripts imprimem (LOG) sai junto; o relatório fica só com as linhas do comparador
    linhas = [l for l in r.stdout.splitlines() if l.startswith(('#', '|', 'linhas', 'Total', 'rand', 'stubs', 'preparo',
                                                                 'casas', 'mapa', 'a missão', 'a coleta', 'ATENÇÃO', '##'))
              or '|' in l or not l.strip()]
    a.relatorio.write_text('\n'.join(linhas).strip() + '\n', encoding='utf-8')
    for l in linhas:
        if l.startswith(('Total', 'rand na', 'casas com', 'preparo pelo')):
            print(l)
    print('relatório em', a.relatorio)


if __name__ == '__main__':
    main()
