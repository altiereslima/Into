#!/usr/bin/env python3
"""Checagem ESTÁTICA de evidências no Breach.exe, sem executar o jogo."""
from pathlib import Path
import hashlib
import struct
import sys
from map_board_bindings import KNOWN_SHA,pe_info,to_offset

CASES={
    0x00561370: [('prologue',bytes.fromhex('55 8B EC')),('size+0x48',bytes.fromhex('3B 41 48')),('size+0x4c',bytes.fromhex('3B 41 4C'))],
    0x00503B60: [('call tile lookup',bytes.fromhex('E8 C2 A0 05 00')),('terrain offset',bytes.fromhex('8B 80 E0 2A 00 00'))],
    0x00567D00: [('terrain==1',bytes.fromhex('83 B8 E0 2A 00 00 01')),('sete al',bytes.fromhex('0F 94 C0'))],
    0x0056FB20: [('width+0x48',bytes.fromhex('8B 41 48')),('height+0x4c',bytes.fromhex('8B 41 4C'))],
    0x00567C50: [('fire+0x2aec',bytes.fromhex('83 BA EC 2A 00 00 00'))],
    0x005653E0: [('smoke+0x2b25',bytes.fromhex('8A 80 25 2B 00 00'))],
    0x005653C0: [('vines+0x2aa0',bytes.fromhex('8A 80 A0 2A 00 00'))],
    0x00567CB0: [('acid+0x2af1',bytes.fromhex('8A 80 F1 2A 00 00'))],
    0x00569920: [('cracked+0x2b26',bytes.fromhex('8A 80 26 2B 00 00'))],
    0x00568950: [('team fallback 2',bytes.fromhex('B8 02 00 00 00')),('pawn team+0xb0',bytes.fromhex('8B 80 B0 00 00 00'))],
    0x0055DC30: [('column table',bytes.fromhex('8B 4E 50')),('row step',bytes.fromhex('69 C3 BC 2B 00 00'))],
    0x004E6260: [('IsPawnSpace virtual dispatch',bytes.fromhex('8B 01 FF 60 50'))],
}

def main():
    file=Path(sys.argv[1]);d=file.read_bytes()
    sha=hashlib.sha256(d).hexdigest()
    if sha!=KNOWN_SHA: raise SystemExit('SHA256 nao corresponde ao EXE base: '+sha)
    sections=pe_info(d)
    count=0
    for va,items in CASES.items():
        window=d[to_offset(sections,va):to_offset(sections,va)+160]
        for label,needle in items:
            assert needle in window, f'Falha: {hex(va)} {label}'
            count+=1
    print('RESULTADO: OK,',len(CASES),'rotinas e',count,'assinaturas de instrucao verificadas.')
    print('NOTA: isso NÃO executa nem valida a semantica do jogo/Rust.')
if __name__=='__main__':main()
