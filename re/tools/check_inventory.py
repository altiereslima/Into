#!/usr/bin/env python3
"""Reproduz e confere inventario nativo com a Etapa 03 original.
Uso: python tools/check_inventory.py /caminho/Breach.exe /caminho/Into.the.Breach.v1.2.88.zip
"""
from pathlib import Path
import csv,hashlib,json,sys,tempfile
from map_all_lua_bindings import EXE_SHA256,build

def run():
 if len(sys.argv)<3:raise SystemExit('Uso: check_inventory.py Breach.exe JOGO.zip')
 data=Path(sys.argv[1]).read_bytes()
 assert hashlib.sha256(data).hexdigest()==EXE_SHA256,'EXE diferente'
 rows,extra=build(data)
 names={r['name'] for r in rows}
 sites=[int(r['registration_va'],16) for r in rows]
 targets={r['native_target_va'] for r in rows}
 full={r['native_target_va'] for r in rows if r['rust_status']=='rust_semantics_written_not_compiled'}
 assert len(rows)==228,(len(rows),'registro inesperado; atualizar baseline com auditoria manual')
 assert len(sites)==len(set(sites)),'registros duplicados'
 assert len(targets)==226
 assert len(full)==9
 assert len(extra)==3
 assert [r['name'] for r in extra]==['desc','money','type']
 assert len([r for r in rows if r['group_provisional']=='Board_provisorio'])==87
 assert {'GetTerrain','GetPawnTeam','IsEdge','IsValid','GetTurnCount'} <= names
 # As 83 entradas ja mapeadas em 03 devem estar presentes na Etapa 04.
 stage3=Path(__file__).resolve().parents[1]/'evidencias'/'board_bindings.csv'
 with stage3.open(encoding='utf-8',newline='') as f:
  old=list(csv.DictReader(f))
 positions={(r['registration_va'],r['name'],r['native_target_va']) for r in rows}
 assert len(old)==83
 for r in old:
  assert (r['registration'],r['method'],r['native_target']) in positions,('regressao Board',r)
 # Integridade do ZIP fonte e dados usados para cruzamento Lua, nao instala jogo.
 import zipfile
 with zipfile.ZipFile(sys.argv[2]) as z:
  assert z.read('Breach.exe')==data,'ZIP e EXE divergentes'
  lua=[n for n in z.namelist() if n.startswith('scripts/') and n.endswith('.lua')]
  assert len(lua)>=150
 print('PASS: SHA256, 228 registros, 226 alvos nativos, 9 alvos em Rust nao testado; 83 entradas Board antigas preservadas; 3 candidatos.')

if __name__=='__main__':run()
