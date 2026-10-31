#!/usr/bin/env python3
"""Inventario COMPLETO (por familia) da API Lua registrada via luabind no Breach.exe v1.2.88.

Etapa 05. Substitui o catalogo de 5 padroes estritos da Etapa 04 (que continua
reproduzido como subconjunto e verificado por tools/check_inventory_v2.py).

Metodo (somente leitura do EXE):
 1. Localiza os construtores de objetos de registro luabind:
    `new` + `[reg]=vftable` + insercao na lista de membros da classe em `[x+0x40]`.
    Cada vftable e tipada pelo RTTI do MSVC (Complete Object Locator em vftable-4),
    o que prova a familia: memfun_registration / property_registration /
    constructor_registration.
 2. Emula a pilha x86 (push/sub/add/lea/mov [ebp+d]) nas funcoes registradoras para
    recuperar, em cada chamada, nome (argumento ou imediato embutido), ponteiro de
    membro (alvo + ajuste de this) e offset de propriedade.
 3. Atribui cada registro a classe Lua aberta mais recentemente (`class_<T>("Nome")`,
    chamada ao construtor base 0x006E3F10).
 4. Localiza funcoes globais (luabind::def em escopo de modulo):
    function_registration com nome/funcao imediatos ou passados por wrapper.
 5. Resolve thunks `vcall` (mov eax,[ecx]; jmp [eax+N]) pela vftable RTTI da classe
    concreta no subobjeto indicado pelo ajuste de this.

Dependencia: `pip install capstone`. Demangling opcional com `llvm-undname` no PATH.
Uso: python tools/map_lua_api_v2.py /caminho/Breach.exe evidencias/
"""
from __future__ import annotations
import collections, csv, hashlib, json, re, shutil, struct, subprocess, sys
from pathlib import Path

try:
    import capstone
except ImportError:  # pragma: no cover
    raise SystemExit('Instale capstone: pip install capstone')

EXE_SHA256 = '4bd9e4cc43140244d7def56d440346be836efc5b072dd3e8b0dda3ff31fc84b0'
OPERATOR_NEW = 0x00753C6F          # chamado por todos os construtores de registro
CLASS_BASE_CTOR = 0x006E3F10
OBJ_ASSIGN_POINT = 0x00450910     # luabind::object::operator=(Point const&) (instanciacao)
OBJ_ASSIGN_COLOR = 0x004509E0     # luabind::object::operator=(GL_Color const&)       # luabind::detail::class_base::class_base(const char*)
VCALL_RE = re.compile(r'^mov eax, dword ptr \[ecx\]$')

# Alvos com semantica Rust escrita E testada com cargo (etapas 01R-05).
# Chave: (classe Lua, nome, VA do alvo nativo resolvido).
RUST_DONE = {
    ('Board', 'IsValid', 0x00561370), ('Board', 'GetTerrain', 0x00503B60),
    ('Board', 'IsBuilding', 0x00567D00), ('Board', 'IsFire', 0x00567C50),
    ('Board', 'IsSmoke', 0x005653E0), ('Board', 'IsVines', 0x005653C0),
    ('Board', 'IsAcid', 0x00567CB0), ('Board', 'IsCracked', 0x00569920),
    ('Board', 'IsEdge', 0x0056FB20),
}
RUST_PARTIAL = {('Board', 'GetPawnTeam', 0x00568950)}
# Preenchido pela Etapa 05 (ver src/board_occupancy.rs). Mantido em arquivo
# separado para o diff entre etapas ficar evidente.
ETAPA05_FILE = Path(__file__).with_name('rust_status.json')


class PE:
    def __init__(self, data: bytes):
        self.data = data
        if data[:2] != b'MZ':
            raise ValueError('Nao e PE (MZ)')
        pe = struct.unpack_from('<I', data, 0x3C)[0]
        if data[pe:pe + 4] != b'PE\0\0' or struct.unpack_from('<H', data, pe + 24)[0] != 0x10B:
            raise ValueError('Somente PE32')
        n = struct.unpack_from('<H', data, pe + 6)[0]
        opt = struct.unpack_from('<H', data, pe + 20)[0]
        self.base = struct.unpack_from('<I', data, pe + 24 + 28)[0]
        self.secs = []
        for i in range(n):
            p = pe + 24 + opt + 40 * i
            name = data[p:p + 8].split(b'\0')[0].decode('ascii', 'replace')
            vsize, rva, rawsize, raw = struct.unpack_from('<IIII', data, p + 8)
            self.secs.append((name, self.base + rva, rawsize, raw))
        self.text = next(s for s in self.secs if s[0] == '.text')
        self.rdata = next(s for s in self.secs if s[0] == '.rdata')

    def off(self, va):
        for _, st, sz, o in self.secs:
            if st <= va < st + sz:
                return o + va - st
        return None

    def va(self, o):
        for _, st, sz, fo in self.secs:
            if fo <= o < fo + sz:
                return st + o - fo
        return None

    def u32(self, va):
        o = self.off(va)
        return None if o is None else struct.unpack_from('<I', self.data, o)[0]

    def cstr(self, va, n=120):
        o = self.off(va)
        if o is None:
            return None
        try:
            s = self.data[o:o + n].split(b'\0')[0].decode('ascii')
        except UnicodeDecodeError:
            return None
        return s or None

    def in_text(self, va):
        return self.text[1] <= va < self.text[1] + self.text[2]


class Analyzer:
    def __init__(self, data: bytes):
        self.pe = PE(data)
        self.d = data
        self.md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
        t = self.pe.text
        self.o0, self.o1 = t[3], t[3] + t[2]
        self._index_calls()
        self._rtti()
        self._retcache = {}

    # ------------------------------------------------------------------ utilidades
    def _index_calls(self):
        self.calls = collections.defaultdict(list)
        base = self.pe.text[1]
        for m in re.finditer(rb'\xe8(.{4})', self.d[self.o0:self.o1], re.S):
            va = base + m.start()
            tgt = (va + 5 + struct.unpack('<i', m.group(1))[0]) & 0xFFFFFFFF
            self.calls[tgt].append(va)

    def fstart(self, va, limit=0x200):
        o = self.pe.off(va)
        for k in range(limit):
            p = o - k
            if self.d[p:p + 3] == b'\x55\x8b\xec' and (self.d[p - 1] in (0xCC, 0xC3) or self.d[p - 3] == 0xC2):
                return va - k
        return None

    def ret_of(self, f):
        if f in self._retcache:
            return self._retcache[f]
        r = None
        o = self.pe.off(f) if f else None
        if o is not None:
            for n, i in enumerate(self.md.disasm(self.d[o:o + 0x3000], f)):
                if i.mnemonic == 'ret':
                    r = int(i.op_str, 16) if i.op_str else 0
                    break
                if n > 1500:
                    break
        self._retcache[f] = r
        return r

    # ------------------------------------------------------------------ RTTI
    def _rtti(self):
        pe, d = self.pe, self.d
        self.tds = {}
        for m in re.finditer(rb'\.\?A[VU][A-Za-z_0-9@?$]+', d):
            v = pe.va(m.start())
            if v:
                self.tds[v - 8] = m.group().decode()
        self.cols = {}
        rd = pe.rdata
        for p in range(rd[3], rd[3] + rd[2] - 20, 4):
            if struct.unpack_from('<I', d, p)[0] == 0:
                td = struct.unpack_from('<I', d, p + 12)[0]
                chd = struct.unpack_from('<I', d, p + 16)[0]
                if td in self.tds and pe.off(chd) is not None:
                    self.cols[pe.va(p)] = (self.tds[td], struct.unpack_from('<I', d, p + 4)[0])
        self.vtables = collections.defaultdict(dict)   # tipo -> {offset_subobjeto: vftable}
        colset = set(self.cols)
        for p in range(rd[3], rd[3] + rd[2] - 4, 4):
            v = struct.unpack_from('<I', d, p)[0]
            if v in colset:
                name, offs = self.cols[v]
                self.vtables[name][offs] = pe.va(p) + 4

    def rtti_type_of_vftable(self, vt):
        col = self.pe.u32(vt - 4)
        return self.cols.get(col, (None,))[0]

    # ------------------------------------------------------------------ construtores
    def find_member_registrars(self):
        """Construtores de memfun/property/constructor_registration."""
        link = re.compile(rb'\x8b[\x48\x49\x4a\x4b\x4e\x4f]\x40\x85\xc9\x75', re.S)
        out = {}
        for m in link.finditer(self.d, self.o0, self.o1):
            va = self.pe.va(m.start())
            fs = self.fstart(va)
            if fs is None or fs in out:
                continue
            body = self.d[self.pe.off(fs):self.pe.off(va)]
            if not any(
                    (self.pe.va(self.pe.off(fs) + c.start()) + 5 + struct.unpack('<i', c.group(1))[0]) & 0xFFFFFFFF == OPERATOR_NEW
                    for c in re.finditer(rb'\xe8(.{4})', body, re.S)):
                continue
            k = None
            for mm in reversed(list(re.finditer(rb'\xc7[\x00-\x03\x06\x07](.{4})', body, re.S))):
                vt = struct.unpack('<I', mm.group(1))[0]
                if self.rtti_type_of_vftable(vt):
                    k = mm; break
            if k is None:
                continue
            vt = struct.unpack('<I', k.group(1))[0]
            rest = body[k.start():]
            mi = re.search(rb'\xc7[\x40-\x43\x46\x47]\x08(.{4})', rest, re.S)
            mo = re.search(rb'\xc7[\x40-\x43\x46\x47]\x0c(.{4})', rest, re.S)
            mfp8 = re.search(rb'\x89[\x40-\x7f]\x14', rest) is not None
            out[fs] = dict(vt=vt, rtti=self.rtti_type_of_vftable(vt),
                           inline_name=struct.unpack('<I', mi.group(1))[0] if mi else None,
                           inline_offset=struct.unpack('<I', mo.group(1))[0] if mo else None,
                           mfp_size=8 if mfp8 else 4, ret=self.ret_of(fs))
        return out

    def find_class_wrappers(self):
        out = {}
        for c in self.calls.get(CLASS_BASE_CTOR, []):
            o = self.pe.off(c)
            mm = list(re.finditer(rb'\x68(.{4})\x8b\xc8', self.d[o - 0x14:o], re.S))
            if not mm:
                continue
            name = self.pe.cstr(struct.unpack('<I', mm[-1].group(1))[0])
            out[self.fstart(c)] = dict(name=name, base_ctor_call=c)
        return out

    # ------------------------------------------------------------------ emulacao
    @staticmethod
    def _ebp(s):
        m = re.search(r'\[ebp ([+-]) (0x[0-9a-f]+|\d+)\]', s)
        if not m:
            return None
        v = int(m.group(2), 0)
        return -v if m.group(1) == '-' else v

    def emulate(self, fs, last, registrars, class_wrappers):
        out = []; stack = []; slots = {}; regs = {}; cur = None
        o = self.pe.off(fs)
        for i in self.md.disasm(self.d[o:o + (last - fs) + 16], fs):
            mn, op = i.mnemonic, i.op_str
            if mn == 'push':
                if re.fullmatch(r'-?(0x[0-9a-f]+|\d+)', op):
                    stack.append(('imm', int(op, 0) & 0xFFFFFFFF))
                else:
                    stack.append(regs.get(op, ('unk', op)))
            elif mn == 'pop':
                if stack: stack.pop()
                regs.pop(op, None)
            elif mn == 'sub' and op.startswith('esp,'):
                stack += [('unk', 'sub')] * (int(op.split(',')[1], 0) // 4)
            elif mn == 'add' and op.startswith('esp,'):
                for _ in range(int(op.split(',')[1], 0) // 4):
                    if stack: stack.pop()
            elif mn == 'lea':
                r, src = [x.strip() for x in op.split(',', 1)]
                dd = self._ebp(src)
                regs[r] = ('slot', dd) if dd is not None else ('unk', op)
            elif mn == 'mov':
                dst, src = [x.strip() for x in op.split(',', 1)]
                if dst.startswith('dword ptr [ebp'):
                    slots[self._ebp(dst)] = int(src, 0) & 0xFFFFFFFF if re.fullmatch(r'0x[0-9a-f]+|\d+', src) else None
                elif dst in ('eax', 'ebx', 'ecx', 'edx', 'esi', 'edi'):
                    if src.startswith('0x'): regs[dst] = ('imm', int(src, 0))
                    elif src in regs: regs[dst] = regs[src]
                    else: regs.pop(dst, None)
            elif mn == 'call':
                try: t = int(op, 0)
                except ValueError: t = None
                if t in registrars:
                    n = (registrars[t]['ret'] or 0) // 4
                    args = list(reversed(stack[-n:])) if n else []
                    res = [('slot', a[1], slots.get(a[1]), slots.get(a[1] + 4)) if a[0] == 'slot' else a for a in args]
                    out.append((i.address, t, res, cur))
                    del stack[len(stack) - n:]
                else:
                    if t in class_wrappers:
                        cur = class_wrappers[t]['name']
                    r = self.ret_of(t) if t else None
                    if r is None:
                        stack = []
                    else:
                        for _ in range(r // 4):
                            if stack: stack.pop()
                regs = {}
        return out

    # ------------------------------------------------------------------ globais
    def find_globals(self):
        gpat = re.compile(rb'\x6a\x14(?:.{0,12})\xe8.{4}.{0,16}?\xc7\x40\x04\x00\x00\x00\x00\xc7\x00(.{4})', re.S)
        g = {}
        for m in gpat.finditer(self.d, self.o0, self.o1):
            vt = struct.unpack('<I', m.group(1))[0]
            rt = self.rtti_type_of_vftable(vt)
            if not rt or 'function_registration' not in rt:
                continue
            fs = self.fstart(self.pe.va(m.start()))
            tail = self.d[m.end():m.end() + 24]
            mn = re.match(rb'\xc7\x40\x08(.{4})\xc7\x40\x0c(.{4})', tail, re.S)
            g[fs] = dict(vt=vt, rtti=rt, name=struct.unpack('<I', mn.group(1))[0] if mn else None,
                         fn=struct.unpack('<I', mn.group(2))[0] if mn else None)
        rows = []
        wrappers = {self.fstart(c): k for k, v in g.items() if v['name'] is None for c in self.calls.get(k, [])}
        site = re.compile(rb'\x68(.{4})\xba(.{4})(?:\xc6\x45\xfc.|\xc7\x45\xfc.{4})?\x8d\x4d.\xe8(.{4})', re.S)
        for m in site.finditer(self.d, self.o0, self.o1):
            callva = self.pe.va(m.end() - 5)
            tgt = (callva + 5 + struct.unpack('<i', m.group(3))[0]) & 0xFFFFFFFF
            if tgt not in wrappers:
                continue
            inner = wrappers[tgt]
            rows.append(dict(call_va=callva, registrar=inner, vt=g[inner]['vt'], rtti=g[inner]['rtti'],
                             name_va=struct.unpack('<I', m.group(2))[0], target=struct.unpack('<I', m.group(1))[0],
                             form='global_name_arg'))
        for k, v in g.items():
            if v['name'] is not None:
                for c in self.calls.get(k, []):
                    rows.append(dict(call_va=c, registrar=k, vt=v['vt'], rtti=v['rtti'], name_va=v['name'],
                                     target=v['fn'], form='global_inline'))
        captured = {r['call_va'] for r in rows}
        missed = [c for w in wrappers for c in self.calls.get(w, []) if c not in captured]
        return rows, missed, g

    # ------------------------------------------------------------------ constantes
    def find_global_values(self):
        """`luabind::globals(L)["NOME"] = valor`.

        Formas encontradas (todas em .text, somente leitura):
          A) operator[](const char*) 0x00450870 com nome empilhado, seguido de lua_pushinteger(imm) embutido;
          B) operator[] + object::operator=(int const&)      0x004508B0  (valor = int em .rdata/.data)
          C) operator[] + object::operator=(Point const&)    0x00450910  (valor = 2 int)
          D) operator[] + object::operator=(GL_Color const&) 0x004509E0  (valor = 4 float)
          E) operator[] + operator=(int local)               0x006802F0  (valor = ultimo `mov [ebp+d],imm`)
        operator[] pode ser 0x00450870 (nome como argumento) ou uma instancia com o nome embutido.
        Leituras (`globals["ANIMS"]` sem atribuicao) ficam como value_type='read_or_unknown'."""
        OBJ_INDEX = 0x00450870
        ASSIGN = {0x004508B0: 'integer_ref', OBJ_ASSIGN_POINT: 'Point', OBJ_ASSIGN_COLOR: 'GL_Color',
                  0x006802F0: 'integer_local', 0x00680B90: 'unsigned_local', 0x00680D00: 'boolean_local',
                  0x00680AE0: 'object_pointer'}
        API = {0x007D253C: 'integer', 0x007D2538: 'number', 0x007D2554: 'boolean', 0x007D2544: 'string'}

        def pushed_name(c):
            o = self.pe.off(c)
            w = self.d[o - 0x20:o]
            for k in range(len(w) - 5, -1, -1):  # busca sobreposta, do mais proximo para tras
                if w[k] != 0x68:
                    continue
                n = self.pe.cstr(struct.unpack('<I', w[k + 1:k + 5])[0])
                if n and re.fullmatch(r'[A-Za-z_][A-Za-z_0-9]*', n):
                    return n
            return None

        def embedded_name(fn):
            o = self.pe.off(fn)
            if o is None or not self.pe.in_text(fn):
                return None
            for m in re.finditer(rb'\x68(.{4})', self.d[o:o + 0x40], re.S):
                n = self.pe.cstr(struct.unpack('<I', m.group(1))[0])
                if n and re.fullmatch(r'[A-Z_][A-Z_0-9]*', n):
                    return n
            return None

        out = {}
        # Formas B..E: partir das atribuicoes
        for asg, kind in ASSIGN.items():
            for c in self.calls.get(asg, []):
                o = self.pe.off(c)
                # chamada ao operator[] imediatamente antes do argumento da atribuicao
                mp = re.search(rb'\xe8(.{4})(?:\x8d\x4d.)?(?:\xc6\x45\xfc.|\xc7\x45\xfc.{4})?(?:\x68.{4}|\x51)'
                               rb'(?:\xc6\x45\xfc.|\xc7\x45\xfc.{4})?\x8b\xc8(?:\xc6\x45\xfc.|\xc7\x45\xfc.{4})?$',
                               self.d[o - 0x24:o], re.S)
                if not mp:
                    continue
                pva = c - 0x24 + mp.start()
                idx = (pva + 5 + struct.unpack('<i', mp.group(1))[0]) & 0xFFFFFFFF
                name = pushed_name(pva) if idx == OBJ_INDEX else embedded_name(idx)
                if not name:
                    continue
                arg = re.search(rb'\x68(.{4})\x8b\xc8(?:\xc6\x45\xfc.|\xc7\x45\xfc.{4})?$', self.d[o - 0x14:o], re.S)
                value = None
                if kind in ('integer_ref', 'Point', 'GL_Color') and arg:
                    so = self.pe.off(struct.unpack('<I', arg.group(1))[0])
                    if kind == 'integer_ref':
                        value = struct.unpack_from('<i', self.d, so)[0]
                    elif kind == 'Point':
                        value = '%d;%d' % struct.unpack_from('<2i', self.d, so)
                    else:
                        value = ';'.join('%g' % v for v in struct.unpack_from('<4f', self.d, so))
                elif kind in ('integer_local', 'unsigned_local', 'boolean_local'):
                    lea = re.search(rb'\x8d\x4d(.)(?:\xc6\x45\xfc.|\xc7\x45\xfc.{4})?\x51\x8b\xc8$', self.d[o - 0x14:o], re.S)
                    if lea:
                        w = self.d[o - 0x400:o]
                        if kind == 'boolean_local':
                            k = w.rfind(b'\xc6\x45' + lea.group(1))
                            if k >= 0:
                                value = 'true' if w[k + 3] else 'false'
                        else:
                            k = w.rfind(b'\xc7\x45' + lea.group(1))
                            if k >= 0:
                                value = struct.unpack('<I' if kind == 'unsigned_local' else '<i', w[k + 3:k + 7])[0]
                elif kind == 'object_pointer':
                    value = 'ponteiro_para_instancia_nativa'
                vt = {'integer_ref': 'integer', 'integer_local': 'integer', 'unsigned_local': 'integer',
                      'boolean_local': 'boolean', 'object_pointer': 'object'}.get(kind, kind)
                out[pva] = dict(name=name, call_va=pva, value_type=vt if value is not None else 'unresolved', value=value)
        # Forma A (e leituras): chamadas a 0x00450870 ainda nao vistas
        cs = sorted(self.calls.get(OBJ_INDEX, []))
        for i, c in enumerate(cs):
            if c in out:
                continue
            o = self.pe.off(c)
            end = cs[i + 1] if i + 1 < len(cs) else c + 0x200
            ins = list(self.md.disasm(self.d[o:o + min(end - c, 0x300)], c))
            vtype = value = None
            for j, x in enumerate(ins):
                if x.mnemonic == 'call' and x.op_str.startswith('dword ptr [0x'):
                    a = int(x.op_str.split('[')[1].rstrip(']'), 16)
                    if a in API:
                        vtype = API[a]
                        pushes = [y for y in ins[max(0, j - 6):j] if y.mnemonic == 'push']
                        if len(pushes) >= 2 and re.fullmatch(r'-?(0x[0-9a-f]+|\d+)', pushes[-2].op_str):
                            value = int(pushes[-2].op_str, 0)
                            if value >= 0x80000000: value -= 1 << 32
                        break
            out[c] = dict(name=pushed_name(c), call_va=c, value_type=vtype or 'read_or_unknown', value=value)
        return [out[k] for k in sorted(out)]

    # ------------------------------------------------------------------ thunks
    def resolve(self, target, adj, cpp_class):
        """Retorna (tipo_alvo, slot, alvo_resolvido)."""
        o = self.pe.off(target)
        ins = list(self.md.disasm(self.d[o:o + 16], target))[:2]
        if len(ins) == 2 and f'{ins[0].mnemonic} {ins[0].op_str}' == 'mov eax, dword ptr [ecx]' and ins[1].mnemonic == 'jmp':
            m = re.fullmatch(r'dword ptr \[eax(?: \+ (0x[0-9a-f]+|\d+))?\]', ins[1].op_str)
            if m:
                slot = int(m.group(1), 0) if m.group(1) else 0
                vt = self.vtables.get(cpp_class, {}).get(adj or 0)
                return 'vcall_thunk', slot, (self.pe.u32(vt + slot) if vt else None)
        if len(ins) >= 1 and ins[0].mnemonic == 'jmp' and ins[0].op_str.startswith('0x'):
            return 'jmp_thunk', None, int(ins[0].op_str, 16)
        return 'direct', None, target

    # ------------------------------------------------------------------ tudo
    def build(self):
        regs = self.find_member_registrars()
        cw = self.find_class_wrappers()
        callers = collections.defaultdict(list)
        for t in regs:
            for c in self.calls.get(t, []):
                callers[self.fstart(c, 0x8000)].append(c)
        rows = []
        for fs, cs in sorted(callers.items()):
            for addr, t, args, cur in self.emulate(fs, max(cs), regs, cw):
                r = regs[t]
                rt = r['rtti']
                kind = ('method' if 'memfun_registration' in rt else 'property' if 'property_registration' in rt
                        else 'constructor' if 'constructor_registration' in rt else 'unknown')
                name_va = r['inline_name'] or (args[0][1] if args and args[0][0] == 'imm' and kind != 'constructor' else None)
                target = adj = offset = None
                if kind == 'method':
                    sl = [a for a in args if a[0] == 'slot']
                    if sl:
                        target = sl[0][2]
                        adj = (sl[0][3] or 0) if r['mfp_size'] == 8 else 0
                if kind == 'property':
                    offset = r['inline_offset'] if r['inline_name'] else (args[1][1] if len(args) > 1 and args[1][0] == 'imm' else None)
                rows.append(dict(kind=kind, lua_class=cur, name=self.pe.cstr(name_va) if name_va else None,
                                 name_va=name_va, call_va=addr, registrar=t, vt=r['vt'], rtti=rt,
                                 name_form='inline' if r['inline_name'] else 'arg', mfp_size=r['mfp_size'],
                                 target=target, this_adjust=adj, prop_offset=offset, registrar_fn=fs))
        grows, missed, g = self.find_globals()
        for gr in grows:
            rows.append(dict(kind='global_function', lua_class=None, name=self.pe.cstr(gr['name_va']),
                             name_va=gr['name_va'], call_va=gr['call_va'], registrar=gr['registrar'], vt=gr['vt'],
                             rtti=gr['rtti'], name_form=gr['form'], mfp_size=4, target=gr['target'],
                             this_adjust=0, prop_offset=None, registrar_fn=self.fstart(gr['call_va'], 0x8000)))
        classes = [dict(kind='class', lua_class=v['name'], name=v['name'], wrapper=k, base_ctor_call=v['base_ctor_call'])
                   for k, v in sorted(cw.items())]
        return rows, classes, missed, regs


def cpp_class_of(rtti):
    m = re.match(r'\.\?AU\?\$(?:memfun|property|constructor)_registration@([UV])([A-Za-z_0-9]+)@@', rtti or '')
    return f'.?A{m.group(1)}{m.group(2)}@@' if m else None


def demangle_all(names):
    tool = shutil.which('llvm-undname') or shutil.which('llvm-undname-18')
    if not tool:
        return {}
    mangled = ['??_7' + n[4:] + '6B@' for n in names]
    p = subprocess.run([tool], input='\n'.join(mangled), capture_output=True, text=True)
    out = {}
    lines = [l for l in p.stdout.splitlines() if l.strip()]
    pairs = [lines[i:i + 2] for i in range(0, len(lines), 2)]
    for n, (_, dem) in zip(names, pairs):
        dem = dem.replace('const ', '', 1).replace("::`vftable'", '')
        out[n] = dem
    return out


def main():
    if len(sys.argv) < 3:
        raise SystemExit('Uso: map_lua_api_v2.py Breach.exe diretorio_saida')
    data = Path(sys.argv[1]).read_bytes()
    sha = hashlib.sha256(data).hexdigest()
    if sha != EXE_SHA256:
        raise SystemExit('SHA256 inesperado: ' + sha)
    dest = Path(sys.argv[2]); dest.mkdir(parents=True, exist_ok=True)
    an = Analyzer(data)
    rows, classes, missed, regs = an.build()
    extra = json.loads(ETAPA05_FILE.read_text(encoding='utf-8')) if ETAPA05_FILE.exists() else {}
    new_done = {(a, b, int(c, 16)) for a, b, c in extra.get('done', [])}
    done = set(RUST_DONE) | new_done
    partial = (set(RUST_PARTIAL) - new_done) | {(a, b, int(c, 16)) for a, b, c in extra.get('partial', [])}
    done -= partial
    trivial = {int(v, 16) for v in extra.get('trivial_accessors', [])}
    dem = demangle_all(sorted({r['rtti'] for r in rows if r['rtti']}))
    out = []
    for r in rows:
        cpp = cpp_class_of(r['rtti'])
        tkind, slot, resolved = (None, None, None)
        if r['target'] is not None:
            tkind, slot, resolved = an.resolve(r['target'], r['this_adjust'], cpp)
        key = (r['lua_class'] or '', r['name'] or '', resolved or -1)
        status = 'not_applicable'
        if r['kind'] in ('method', 'global_function'):
            status = ('rust_partial' if key in partial else 'rust_semantics_tested' if key in done
                      else 'native_located_not_reconstructed')
            if resolved is None:
                status = 'native_unresolved_virtual'
        h = lambda v: '' if v is None else f'0x{v:08X}'
        out.append({
            'kind': r['kind'], 'lua_class': r['lua_class'] or '', 'name': r['name'] or '',
            'registration_call_va': h(r['call_va']), 'registrar_va': h(r['registrar']),
            'registration_vftable_va': h(r['vt']), 'name_form': r['name_form'], 'name_string_va': h(r['name_va']),
            'native_target_va': h(r['target']), 'this_adjust': '' if r['this_adjust'] is None else r['this_adjust'],
            'target_kind': tkind or '', 'vtable_slot_offset': '' if slot is None else f'0x{slot:X}',
            'resolved_target_va': h(resolved), 'property_offset': '' if r['prop_offset'] is None else f'0x{r["prop_offset"]:X}',
            'cpp_registration_rtti': r['rtti'] or '', 'cpp_signature_demangled': dem.get(r['rtti'], ''),
            'rust_status': status})
    out.sort(key=lambda x: int(x['registration_call_va'], 16))

    def save(name, items):
        with (dest / name).open('w', newline='', encoding='utf-8') as f:
            w = csv.DictWriter(f, fieldnames=list(items[0].keys())); w.writeheader(); w.writerows(items)
    save('lua_api_v2_all.csv', out)
    cppmap = {}
    for r in rows:
        if r['kind'] != 'global_function' and r['lua_class']:
            cppmap.setdefault(r['lua_class'], cpp_class_of(r['rtti']))
    count = collections.Counter((o['lua_class'], o['kind']) for o in out)
    save('lua_api_v2_classes.csv', [{'lua_class': c['name'], 'cpp_class_rtti': cppmap.get(c['name']) or '',
                                    'class_wrapper_va': f'0x{c["wrapper"]:08X}',
                                    'base_ctor_call_va': f'0x{c["base_ctor_call"]:08X}',
                                    'methods': count[(c['name'], 'method')], 'properties': count[(c['name'], 'property')],
                                    'constructors': count[(c['name'], 'constructor')]} for c in classes])
    gvals = an.find_global_values()
    save('lua_global_values.csv', [{'name': g['name'] or '', 'operator_index_call_va': f'0x{g["call_va"]:08X}',
                                    'value_type': g['value_type'], 'value': '' if g['value'] is None else g['value']}
                                   for g in gvals])

    fn_rows = [o for o in out if o['kind'] in ('method', 'global_function')]
    ent = {o['resolved_target_va'] or ('UNRES:' + o['native_target_va'] + ':' + o['vtable_slot_offset']) for o in fn_rows}
    st = collections.defaultdict(set)
    for o in fn_rows:
        st[o['rust_status']].add(o['resolved_target_va'] or o['native_target_va'])
    n_ent = len(ent); n_done = len(st['rust_semantics_tested']); n_part = len(st['rust_partial'])
    kinds = collections.Counter(o['kind'] for o in out)
    stats = {
        'stage': '05', 'source_sha256': sha,
        'classes_lua': len(classes),
        'registrations_by_kind': dict(sorted(kinds.items())),
        'callable_registrations_methods_plus_globals': len(fn_rows),
        'callable_registrations_tested_in_rust': sum(1 for o in fn_rows if o['rust_status'] == 'rust_semantics_tested'),
        'distinct_native_entrypoints_resolved': n_ent,
        'entrypoints_rust_semantics_tested': n_done,
        'entrypoints_rust_partial': n_part,
        'entrypoints_remaining_including_partial': n_ent - n_done,
        'percent_entrypoints_reconstructed': round(100 * n_done / n_ent, 2),
        'percent_entrypoints_remaining': round(100 * (n_ent - n_done) / n_ent, 2),
        'entrypoints_tested_trivial_accessors': len({v for v in st['rust_semantics_tested'] if v and int(v, 16) in trivial}),
        'entrypoints_tested_with_logic': n_done - len({v for v in st['rust_semantics_tested'] if v and int(v, 16) in trivial}),
        'globals_operator_index_sites': len(gvals),  # inclui leituras (ex.: ANIMS) e atribuicoes de objetos
        'global_integer_constants': sum(1 for g in gvals if g['value_type'] == 'integer'),
        'unresolved_virtual_targets': sum(1 for o in fn_rows if o['rust_status'] == 'native_unresolved_virtual'),
        'global_wrapper_calls_not_captured': [f'0x{c:08X}' for c in missed],
        'member_registrars_found': len(regs),
        'registrar_functions': sorted({f'0x{r["registrar_fn"]:08X}' for r in rows if r['registrar_fn']}),
        'etapa04_strict_subset_rows': 228,
        'scope': ('Registros luabind (classes, metodos, propriedades, construtores, funcoes globais) '
                  'achados por tipagem RTTI + emulacao de pilha. Fora do escopo: metametodos internos do '
                  'luabind e tabelas/constantes Lua criadas pelos proprios scripts.'),
    }
    (dest / 'lua_api_v2_progress.json').write_text(json.dumps(stats, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(stats, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
