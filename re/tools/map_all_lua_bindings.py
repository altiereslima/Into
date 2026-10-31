#!/usr/bin/env python3
"""Catalogo progressivo e conservador da API Lua no Breach.exe v1.2.88.

NAO assume que encontrou todas as funcoes. Regex cobre duas formas x86
verificadas manualmente, mais lista separada de candidatos.
Nao altera EXE, scripts nem o projeto Rust.

Uso: python tools/map_all_lua_bindings.py /caminho/Breach.exe evidencias/
"""
from __future__ import annotations
import csv, hashlib, json, re, struct, sys
from collections import Counter
from pathlib import Path

EXE_SHA256 = '4bd9e4cc43140244d7def56d440346be836efc5b072dd3e8b0dda3ff31fc84b0'
# Janela de registro identificada a partir de sequencias de construcao luabind,
# NAO um limite exaustivo do programa. Varredura das formas estritas ocorre em
# TODA a secao .text, incluindo outras partes do EXE.
CANDIDATE_LO, CANDIDATE_HI = 0x00676000, 0x00679B00
PATTERNS = {
  'native_with_flags': re.compile(rb'\xc7\x45(?P<off>.)(?P<target>.{4})\xff\x75\xf0\xc7\x45(?P<flagoff>.)(?P<flags>.{4})\x51\x8d\x4d(?P=off)\x51\x68(?P<name>.{4})\x8b\xc8\xe8(?P<call>.{4})',re.S),
  'native_simple': re.compile(rb'\xff\x75\xf0\xc7\x45(?P<off>.)(?P<target>.{4})\xff\x75\xf0\x51\x8d\x4d(?P=off)\x51\x68(?P<name>.{4})\x8b\xc8\xe8(?P<call>.{4})',re.S),
  'native_flags_extra_push': re.compile(rb'\xc7\x45(?P<off>.)(?P<target>.{4})\xff\x75\xf0\xc7\x45(?P<flagoff>.)(?P<flags>.{4})\xff\x75\xf0\x51\x8d\x4d(?P=off)\x51\x68(?P<name>.{4})\x8b\xc8\xe8(?P<call>.{4})',re.S),
  'native_simple_extra_push': re.compile(rb'\xc7\x45(?P<off>.)(?P<target>.{4})\xff\x75\xf0\xff\x75\xf0\x51\x8d\x4d(?P=off)\x51\x68(?P<name>.{4})\x8b\xc8\xe8(?P<call>.{4})',re.S),
  'native_flags_double_push': re.compile(rb'\xc7\x45(?P<off>.)(?P<target>.{4})\xc7\x45(?P<flagoff>.)(?P<flags>.{4})\xff\x75\xf0\xff\x75\xf0\x51\x8d\x4d(?P=off)\x51\x68(?P<name>.{4})\x8b\xc8\xe8(?P<call>.{4})',re.S),
}
OTHER_CALL = re.compile(rb'\x68(?P<name>.{4})\x8b\xc8\xe8(?P<call>.{4})',re.S)
VALID_NAME = re.compile(r'^[A-Za-z_][A-Za-z_0-9]{1,60}$')
# Somente alvos para os quais etapas anteriores documentaram semantica e
# escreveram Rust. Nao significa que cargo test foi executado.
RUST_SEMANTICS = {
 'IsValid': {0x00561370}, 'GetTerrain': {0x00503B60},
 'IsBuilding': {0x00567D00}, 'IsFire': {0x00567C50},
 'IsSmoke': {0x005653E0}, 'IsVines': {0x005653C0},
 'IsAcid': {0x00567CB0}, 'IsCracked': {0x00569920},
 'IsEdge': {0x0056FB20},
}
PARTIAL = {'GetPawnTeam': {0x00568950}}

def u32(b: bytes) -> int:
 return struct.unpack('<I',b)[0]

def sections(data:bytes):
 if data[:2] != b'MZ': raise ValueError('Nao e PE (MZ)')
 pe=u32(data[0x3c:0x40]);
 if data[pe:pe+4] != b'PE\0\0': raise ValueError('Assinatura PE invalida')
 if struct.unpack_from('<H',data,pe+24)[0] != 0x10b: raise ValueError('Somente PE32')
 count=struct.unpack_from('<H',data,pe+6)[0]
 optsize=struct.unpack_from('<H',data,pe+20)[0]
 base=u32(data[pe+24+28:pe+24+32]); out=[]
 for i in range(count):
  p=pe+24+optsize+40*i
  name=data[p:p+8].split(b'\0')[0].decode('ascii','replace')
  vsize,rva,rawsize,raw=struct.unpack_from('<IIII',data,p+8)
  out.append((name,base+rva,rawsize,raw))
 return out

def to_file(secs,va):
 for _,start,size,off in secs:
  if start <= va < start+size:return off+va-start
 raise ValueError('Endereco nao mapeado 0x%X'%va)

def read_name(data,secs,ptr):
 try:
  o=to_file(secs,ptr)
  s=data[o:o+100].split(b'\0')[0].decode('ascii')
  return s if VALID_NAME.fullmatch(s) else None
 except (ValueError,UnicodeDecodeError):return None

def group(va):
 if 0x00676000<=va<0x0067625E:return 'conteiners_auxiliares_provisorio'
 if 0x0067625E<=va<0x00677500:return 'Board_provisorio'
 if 0x00677500<=va<0x00678460:return 'outros_tipos_provisorio'
 if 0x00678460<=va<0x00679000:return 'Pawn_provisorio'
 if 0x00679000<=va<0x00679B00:return 'Game_e_outros_provisorio'
 return 'fora_da_janela_inicial'

def build(data:bytes):
 secs=sections(data)
 text=[s for s in secs if s[0]=='.text'][0]
 _,base,size,off=text
 covered=[]
 rows=[]
 for kind,pat in PATTERNS.items():
  for m in pat.finditer(data,off,off+size):
   va=base+m.start()-off
   name_va=u32(m.group('name'))
   name=read_name(data,secs,name_va)
   if not name:continue
   target=u32(m.group('target'))
   site=base+m.end()-12-off # opcode 68 + imm32; mov ecx,eax; call rel32
   call=base+m.end()-5-off
   rel=struct.unpack('<i',m.group('call'))[0]
   bind=call+5+rel
   status='native_located_not_reconstructed'
   if target in RUST_SEMANTICS.get(name,set()):status='rust_semantics_written_not_compiled'
   if target in PARTIAL.get(name,set()):status='rust_partial'
   rows.append({'name':name,'registration_va':f'0x{va:08X}',
    'name_push_va':f'0x{site:08X}', 'name_string_va':f'0x{name_va:08X}',
    'native_target_va':f'0x{target:08X}',
    'binding_helper_va':f'0x{bind:08X}', 'pattern':kind,
    'group_provisional':group(va),'rust_status':status})
   covered.append(site)
 rows.sort(key=lambda r:int(r['registration_va'],16))
 # Qualquer novo candidato aguarda confirmacao manual. Nao entra no denominador!
 candidates=[]
 for m in OTHER_CALL.finditer(data,off,off+size):
  va=base+m.start()-off
  if not (CANDIDATE_LO<=va<CANDIDATE_HI):continue
  name_va=u32(m.group('name'))
  name=read_name(data,secs,name_va)
  if name and va not in covered:
   candidates.append({'name':name,'name_push_va':f'0x{va:08X}',
    'name_string_va':f'0x{name_va:08X}', 'group_provisional':group(va),
    'status':'candidate_needs_manual_disassembly'})
 return rows,candidates

def main():
 if len(sys.argv)<3:raise SystemExit('Uso: map_all_lua_bindings.py Breach.exe diretorio_saida')
 exe=Path(sys.argv[1]);dest=Path(sys.argv[2]);data=exe.read_bytes()
 sha=hashlib.sha256(data).hexdigest()
 if sha!=EXE_SHA256:raise SystemExit('SHA256 inesperado: '+sha)
 rows,candidates=build(data)
 if len(rows)<228 or len(candidates)<3:
  raise SystemExit(f'Regressao no parser: {len(rows)} registros, {len(candidates)} candidatos')
 dest.mkdir(parents=True,exist_ok=True)
 def save_csv(path,items):
  if not items:return
  with (dest/path).open('w',newline='',encoding='utf-8') as f:
   w=csv.DictWriter(f,fieldnames=list(items[0].keys()));w.writeheader();w.writerows(items)
 save_csv('lua_bindings_confirmed.csv',rows)
 save_csv('lua_bindings_candidates.csv',candidates)
 counts=Counter(r['rust_status'] for r in rows)
 total=len(rows);rust=counts['rust_semantics_written_not_compiled'];partial=counts['rust_partial'];missing=total-rust
 unique_targets={r['native_target_va'] for r in rows}
 rust_targets={r['native_target_va'] for r in rows if r['rust_status']=='rust_semantics_written_not_compiled'}
 partial_targets={r['native_target_va'] for r in rows if r['rust_status']=='rust_partial'}
 stats={'source_sha256':sha,'confirmed_lower_bound':total,
  'candidate_additional_unverified':len(candidates),
  'unique_native_entrypoints':len(unique_targets),
  'unique_entrypoints_with_rust_semantics':len(rust_targets),
  'unique_entrypoints_partial':len(partial_targets),
  'unique_entrypoints_remaining_including_partial':len(unique_targets-rust_targets),
  'percent_unique_entrypoints_reconstructed':round(100*len(rust_targets)/len(unique_targets),2),
  'percent_unique_entrypoints_remaining':round(100*len(unique_targets-rust_targets)/len(unique_targets),2),
  'rust_semantics_written_not_compiled':rust,
  'rust_partial':partial,'not_yet_reconstructed_including_partial':missing,
  'percent_rust_semantics_written':round(100*rust/total,2),
  'percent_missing_rust':round(100*missing/total,2),
  'global_registration_coverage_percent':None,
  'global_remaining_percent':None,
  'disclaimer':'Denominador MINIMO, ainda nao inventario exaustivo; pode crescer. Rust nao compilado.',
  'groups_provisional':dict(sorted(Counter(r['group_provisional'] for r in rows).items())),
  'patterns':dict(sorted(Counter(r['pattern'] for r in rows).items()))}
 (dest/'lua_progress.json').write_text(json.dumps(stats,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(stats,ensure_ascii=False,indent=2))

if __name__=='__main__':main()
