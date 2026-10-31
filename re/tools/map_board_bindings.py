#!/usr/bin/env python3
"""Extrai registros Board via padrão estrito observado no Breach.exe 1.2.88.
Uso: python tools/map_board_bindings.py /caminho/Breach.exe [saida.csv]
Nao modifica o executavel. Um registro nao e equivalencia semantica de uma funcao.
"""
import csv
import hashlib
from pathlib import Path
import re
import struct
import sys

KNOWN_SHA = '4bd9e4cc43140244d7def56d440346be836efc5b072dd3e8b0dda3ff31fc84b0'
START = 0x00676240
STOP = 0x00677500  # faixa do construtor da tabela Board; limites a confirmar


def pe_info(data):
    if data[:2] != b'MZ': raise ValueError('Not MZ')
    pe = struct.unpack_from('<I', data, 0x3c)[0]
    if data[pe:pe+4] != b'PE\0\0': raise ValueError('Not PE')
    count = struct.unpack_from('<H', data, pe+6)[0]
    opt = pe+24
    if struct.unpack_from('<H', data, opt)[0] != 0x10b: raise ValueError('PE32 only')
    base = struct.unpack_from('<I', data, opt+28)[0]
    optional_size=struct.unpack_from('<H', data, pe+20)[0]
    sections=[]
    for i in range(count):
        p=opt+optional_size+40*i
        name=data[p:p+8].split(b'\0')[0].decode('ascii','replace')
        vsize, va, rawsize, raw=struct.unpack_from('<IIII', data, p+8)
        sections.append((name,base+va, max(vsize,rawsize),raw,rawsize))
    return sections


def to_offset(sections,va):
    for name,start,size,off,raw_size in sections:
        delta=va-start
        if 0 <= delta < raw_size: return off+delta
    raise ValueError('Invalid VA 0x%x'%va)


def main():
    p=Path(sys.argv[1]); data=p.read_bytes()
    sha=hashlib.sha256(data).hexdigest()
    if sha != KNOWN_SHA:
        raise SystemExit('SHA256 diferente da base validada; interrompendo. Obtido '+sha)
    sections=pe_info(data)
    lo=to_offset(sections,START);hi=to_offset(sections,STOP)
    # mov [ebp-18],function; push [ebp-10]; mov [ebp-14],arg_flag;
    # push ecx; lea ecx,[ebp-18]; push ecx; push name; mov ecx,eax; call binder
    pattern=re.compile(rb'\xc7\x45\xe8(.{4})\xff\x75\xf0\xc7\x45\xec(.{4})\x51\x8d\x4d\xe8\x51\x68(.{4})\x8b\xc8\xe8(.{4})',re.DOTALL)
    rows=[]
    for match in pattern.finditer(data,lo,hi):
        target,flags,name,relative=[struct.unpack('<I',match.group(i))[0] for i in range(1,5)]
        loc=START+(match.start()-lo)
        call_offset=match.end()-5
        binder=(START+(call_offset-lo)+5+(relative if relative<0x80000000 else relative-0x100000000)) & 0xffffffff
        try:
            s_off=to_offset(sections,name)
            method=data[s_off:s_off+120].split(b'\0')[0].decode('ascii')
        except (ValueError,UnicodeDecodeError):continue
        if not method or not re.fullmatch(r'[A-Za-z0-9_]{2,60}',method):continue
        rows.append({'method':method,'native_target':f'0x{target:08X}',
                     'registration':f'0x{loc:08X}', 'name_va':f'0x{name:08X}',
                     'binding_helper':f'0x{binder:08X}','flags_raw':f'0x{flags:08X}',
                     'kind':'direct_pointer_or_trampoline'})
    out=Path(sys.argv[2]) if len(sys.argv)>2 else Path('board_bindings.csv')
    with out.open('w',newline='',encoding='utf-8') as f:
        w=csv.DictWriter(f,fieldnames=list(rows[0]) if rows else ['method'])
        w.writeheader();w.writerows(rows)
    print('SOURCE SHA256',sha,'\nREGISTROS',len(rows),'\nSAIDA',out)
    for row in rows[:15]: print(row['method'],row['native_target'],row['registration'])
    if len(rows) < 45: raise SystemExit('ERRO: mapeamento excessivamente pequeno')


if __name__=='__main__':main()
