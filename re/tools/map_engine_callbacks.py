#!/usr/bin/env python3
"""API REVERSA: funcoes Lua que o motor C++ chama (callbacks), para o marco M3.

Metodo (somente leitura):
 1. Coleta nomes de funcoes globais e de metodos DEFINIDOS nos scripts originais.
 2. Procura cada nome como string terminada em zero no .rdata do EXE.
 3. Mantem apenas strings com referencia no .text FORA das funcoes registradoras do luabind
    (onde os nomes sao de bindings, nao de callbacks).
 4. Para cada referencia, registra a funcao C++ que a contem e quais wrappers de chamada Lua ela usa
    (`call_function` -> lua_getfield(L, LUA_GLOBALSINDEX, nome)).

E um inventario de CANDIDATOS: a presenca do nome perto de uma chamada Lua e forte indicio, mas o
contrato exato (argumentos/retorno) de cada callback precisa ser lido no codigo.

Uso: python re/tools/map_engine_callbacks.py Breach.exe pasta/scripts re/evidencias/
"""
import collections, csv, hashlib, os, re, struct, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
import map_lua_api_v2 as m

GETFIELD_IAT = 0x007D24E0
GLOBALSINDEX = 0xFFFFD8EE  # -10002


def scan_scripts(root):
    glob, meth = {}, collections.defaultdict(set)
    for dp, _, fs in os.walk(root):
        for f in sorted(fs):
            if not f.endswith('.lua'):
                continue
            rel = os.path.relpath(os.path.join(dp, f), os.path.dirname(root)).replace('\\', '/')
            t = open(os.path.join(dp, f), encoding='utf-8', errors='replace').read()
            for mm in re.finditer(r'^\s*function\s+([A-Za-z_]\w*)\s*\(', t, re.M):
                glob.setdefault(mm.group(1), rel)
            for mm in re.finditer(r'^\s*([A-Za-z_]\w*)\s*=\s*function', t, re.M):
                glob.setdefault(mm.group(1), rel)
            for mm in re.finditer(r'function\s+([A-Za-z_]\w*)[:.]([A-Za-z_]\w*)\s*\(', t):
                meth[mm.group(2)].add(mm.group(1))
    return glob, meth


def main():
    exe, scripts, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    data = exe.read_bytes()
    assert hashlib.sha256(data).hexdigest() == m.EXE_SHA256
    an = m.Analyzer(data)
    rows, classes, missed, regs = an.build()
    registrar_fns = {r['registrar_fn'] for r in rows if r['registrar_fn']} | set(regs) | {r['registrar'] for r in rows}
    # wrappers de chamada Lua: funcoes que chamam lua_getfield(L, GLOBALSINDEX, ...)
    iat = struct.pack('<I', GETFIELD_IAT)
    wrappers = set()
    for mm in re.finditer(b'\xff\x15' + re.escape(iat), data[an.o0:an.o1]):
        va = an.pe.text[1] + mm.start()
        win = data[an.pe.off(va) - 0x14:an.pe.off(va)]
        if struct.pack('<I', GLOBALSINDEX) in win:
            fs = an.fstart(va, 0x400)
            if fs:
                wrappers.add(fs)
    # strings do .rdata
    rd = an.pe.rdata
    seg = data[rd[3]:rd[3] + rd[2]]
    strs = {}
    for mm in re.finditer(rb'[A-Za-z_][A-Za-z_0-9]{2,60}\x00', seg):
        if mm.start() == 0 or seg[mm.start() - 1] == 0:
            strs.setdefault(mm.group()[:-1].decode(), rd[1] + mm.start())
    glob, meth = scan_scripts(scripts)
    res = []
    for kind, names in (('global_function', glob), ('method', meth)):
        for name in sorted(names):
            if name not in strs or (kind == 'method' and name in glob):
                continue
            sva = strs[name]
            pat = struct.pack('<I', sva)
            for x in re.finditer(re.escape(pat), data[an.o0:an.o1]):
                ref = an.pe.text[1] + x.start()
                fs = an.fstart(ref, 0x8000)
                if fs in registrar_fns:
                    continue
                body_calls = set()
                if fs:
                    o = an.pe.off(fs)
                    for c in re.finditer(rb'\xe8(.{4})', data[o:an.pe.off(ref) + 0x40], re.S):
                        t = (fs + c.start() + 5 + struct.unpack('<i', c.group(1))[0]) & 0xFFFFFFFF
                        if t in wrappers:
                            body_calls.add(t)
                res.append({'kind': kind, 'name': name,
                            'defined_in': glob.get(name, '') if kind == 'global_function' else ';'.join(sorted(meth[name])[:6]),
                            'string_va': f'0x{sva:08X}', 'ref_va': f'0x{ref:08X}',
                            'cpp_function_va': f'0x{fs:08X}' if fs else '',
                            'lua_call_wrappers_in_function': ';'.join(f'0x{t:08X}' for t in sorted(body_calls)),
                            'status': 'candidate_callback'})
    out.mkdir(parents=True, exist_ok=True)
    with (out / 'engine_callbacks.csv').open('w', newline='', encoding='utf-8') as f:
        w = csv.DictWriter(f, fieldnames=list(res[0].keys())); w.writeheader(); w.writerows(res)
    names = collections.Counter((r['kind'], r['name']) for r in res)
    print(f'{len(wrappers)} wrappers de chamada Lua; {len(names)} callbacks candidatos '
          f'({sum(1 for k in names if k[0] == "global_function")} funcoes globais, '
          f'{sum(1 for k in names if k[0] == "method")} metodos) em {len(res)} referencias')


if __name__ == '__main__':
    main()
