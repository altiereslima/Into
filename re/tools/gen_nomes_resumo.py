#!/usr/bin/env python3
"""Junta nomes_android.csv + nomes_manual.csv e gera re/evidencias/nomes_android_resumo.md com os nomes das
funções citadas no código e nas evidências. Uso: python re/tools/gen_nomes_resumo.py (na raiz do repositório)."""
import csv, glob, re
from collections import Counter

auto = list(csv.DictReader(open('re/evidencias/nomes_android.csv', encoding='utf-8')))
man = list(csv.DictReader(open('re/evidencias/nomes_manual.csv', encoding='utf-8')))
names = {int(r['pc_va'], 16): (r['nome'], r['metodo'], r['confianca']) for r in auto}
for r in man:
    names[int(r['pc_va'], 16)] = (r['nome'].replace(';', ','), 'manual', r['confianca'])
new_va = {int(r['va_1288'], 16): r['va_1293'] for r in csv.DictReader(open('re/evidencias/v1.2.93/migracao_enderecos.csv', encoding='utf-8'))}
cited = Counter()
# api_manifest.rs traz endereços da 1.2.93; os alvos da API entram pelo inventário da 1.2.88.
for r in csv.DictReader(open('re/evidencias/lua_api_v2_all.csv', encoding='utf-8')):
    if r['resolved_target_va']:
        cited[int(r['resolved_target_va'], 16)] += 1
for f in glob.glob('crates/**/*.rs', recursive=True) + glob.glob('re/evidencias/*.md'):
    if f.endswith(('nomes_android_resumo.md', 'api_manifest.rs')):
        continue
    for m in re.finditer(r'0x00([4-7][0-9A-Fa-f]{5})\b', open(f, encoding='utf-8').read()):
        cited[int(m.group(1), 16)] += 1
have = sorted(a for a in cited if a in names)
out = ["# Nomes das funções do PC vindos do Android (libmain.so 1.2.92)", "",
       "Gerado por `re/tools/match_android_symbols.py` + `nomes_manual.csv` (`re/tools/gen_nomes_resumo.py`).",
       "O `libmain.so` fica no PC do usuário, fora do repositório; aqui só há nomes e endereços.",
       "A coluna 1.2.93 vem de `v1.2.93/migracao_enderecos.csv`.", "",
       f"* casamentos automáticos: {len(auto)} ({dict(Counter(r['metodo'] for r in auto))}); manuais: {len(man)}",
       f"* endereços de código citados no projeto: {len(cited)}; com nome agora: {len(have)}", "",
       "## Validação", "Cada sinal foi conferido contra os outros: vftable × Lua 8/9; texto × Lua/vftable 62/65;",
       "vizinhos (deixando cada casamento de fora) 68/69. Alinhar a ordem das chamadas falhou (0/72) e não é usado.",
       "Confiança `media` = sinal mais fraco (1 string, nome Lua aproximado, vizinhos ou leitura manual).", "",
       "## Achados",
       "* A casa do tabuleiro é `BoardSpace`; a base de busca é `GridSearchable` (Dijkstra = `CalculateDijkstraMap`, A* = `GetAstarPath`).",
       "* `Pawn::IsMassive()` existe; o byte +0x99D do PC (água não afoga, gelo danificado não mata) cumpre esse papel (inlinado no PC).",
       "* Fila de efeitos: `Board::AddEffect` → `StackEffect` → `UpdateStackedEffects` → `ApplyEffect` (executor).", "",
       "## Funções citadas no projeto", "", "| PC 1.2.88 | PC 1.2.93 | Nome no Android | Como | Confiança |", "|---|---|---|---|---|"]
for a in have:
    n, how, c = names[a]
    out.append(f"| 0x{a:08X} | {new_va.get(a, '')} | `{n}` | {how} | {c} |")
open('re/evidencias/nomes_android_resumo.md', 'w', encoding='utf-8', newline='\n').write('\n'.join(out) + '\n')
print(f'{len(have)} de {len(cited)} endereços citados com nome')
