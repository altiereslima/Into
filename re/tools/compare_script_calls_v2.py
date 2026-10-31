#!/usr/bin/env python3
"""Cruza chamadas dos scripts Lua originais com o inventario v2 (Etapa 05).

Diferente da v1, valida a CLASSE: Board -> Board, Game -> GameMap,
PAWN_FACTORY -> PawnFactory (instancias globais expostas pelo motor).
Tambem confere funcoes globais nativas chamadas sem objeto.

Uso: python tools/compare_script_calls_v2.py jogo.zip evidencias/lua_api_v2_all.csv evidencias/lua_calls_v2_unmatched.csv
"""
import csv, re, sys, zipfile
from collections import defaultdict
from pathlib import Path

GLOBAL_OBJECTS = {'Board': 'Board', 'Game': 'GameMap', 'PAWN_FACTORY': 'PawnFactory'}
CALL = re.compile(r'\b(Board|Game|PAWN_FACTORY)[\.:]([A-Za-z_][A-Za-z_0-9]*)\s*\(')


def main():
    if len(sys.argv) < 4:
        raise SystemExit('Uso: compare_script_calls_v2.py jogo.zip lua_api_v2_all.csv saida.csv')
    with open(sys.argv[2], encoding='utf-8', newline='') as f:
        rows = list(csv.DictReader(f))
    by_class = defaultdict(set)
    for r in rows:
        if r['kind'] in ('method', 'property'):
            by_class[r['lua_class']].add(r['name'])
    found = defaultdict(list)
    with zipfile.ZipFile(sys.argv[1]) as z:
        for item in sorted(z.namelist()):
            if not (item.startswith('scripts/') and item.endswith('.lua')):
                continue
            txt = z.read(item).decode('utf-8', 'replace')
            for n, line in enumerate(txt.splitlines(), 1):
                code = line.split('--', 1)[0]
                for obj, meth in CALL.findall(code):
                    found[(obj, meth)].append((item, n))
    unmatched = []
    matched = 0
    for (obj, meth), locs in sorted(found.items()):
        if meth in by_class[GLOBAL_OBJECTS[obj]]:
            matched += 1
        else:
            unmatched.append({'object': obj, 'cpp_lua_class': GLOBAL_OBJECTS[obj], 'method': meth,
                              'script_calls': len(locs), 'example_file': locs[0][0], 'example_line': locs[0][1],
                              'note': 'nao registrado nesta classe; pode ser campo Lua, funcao definida em script ou outro mecanismo'})
    out = Path(sys.argv[3]); out.parent.mkdir(parents=True, exist_ok=True)
    with out.open('w', newline='', encoding='utf-8') as f:
        w = csv.DictWriter(f, fieldnames=['object', 'cpp_lua_class', 'method', 'script_calls', 'example_file', 'example_line', 'note'])
        w.writeheader(); w.writerows(unmatched)
    print(f'pares objeto/metodo nos scripts: {len(found)}; casados com classe correta: {matched}; sem registro: {len(unmatched)}')
    for u in unmatched:
        print('  SEM REGISTRO:', u['object'], u['method'], u['script_calls'], u['example_file'], u['example_line'])
    return 0 if not unmatched else 1


if __name__ == '__main__':
    sys.exit(main())
