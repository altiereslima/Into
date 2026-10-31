#!/usr/bin/env python3
"""Cruza scripts Lua originais com registros nativos confirmados.

Uso: python tools/compare_script_calls.py /caminho/Into.the.Breach.v1.2.88.zip evidencias/lua_bindings_confirmed.csv evidencias/lua_calls_without_binding.csv
O cruzamento apenas por nome produz CANDIDATOS, nao provas de classe ou registro.
"""
import csv,re,sys,zipfile
from collections import Counter,defaultdict
from pathlib import Path

CALL=re.compile(r'\b(Board|Game|PAWN_FACTORY)[\.:]([A-Za-z_][A-Za-z_0-9]*)\s*\(')

def main():
 if len(sys.argv)<4:raise SystemExit('Uso: compare_script_calls.py jogo.zip bindings.csv resultado.csv')
 with open(sys.argv[2],encoding='utf-8',newline='') as f:
  known={r['name'] for r in csv.DictReader(f)}
 found=defaultdict(list)
 with zipfile.ZipFile(sys.argv[1]) as z:
  for item in sorted(z.namelist()):
   if not item.startswith('scripts/') or not item.endswith('.lua'):continue
   txt=z.read(item).decode('utf-8','replace')
   for lineno,line in enumerate(txt.splitlines(),1):
    for obj,method in CALL.findall(line):
     found[(obj,method)].append((item,lineno))
 rows=[]
 for (obj,method),locs in found.items():
  if method not in known:
   rows.append({'object':obj,'method':method,'script_calls':len(locs),
       'example_file':locs[0][0], 'example_line':locs[0][1],
       'caveat':'nao encontrado no catalogo direto; pode ser overload, outro padrao ou Lua puro'})
 rows.sort(key=lambda row:(-row['script_calls'],row['object'],row['method']))
 output=Path(sys.argv[3]);output.parent.mkdir(parents=True,exist_ok=True)
 with output.open('w',newline='',encoding='utf-8') as f:
  w=csv.DictWriter(f,fieldnames=['object','method','script_calls','example_file','example_line','caveat'])
  w.writeheader();w.writerows(rows)
 print('API pairs scripts:',len(found),'nome presente no registro:',len(found)-len(rows),'nao localizado pelo nome:',len(rows))
 print('ALVOS:',[(x['object'],x['method'],x['script_calls']) for x in rows[:10]])

if __name__=='__main__':main()
