#!/usr/bin/env python3
"""Dá nome às funções do Breach.exe (PC 1.2.88, x86, sem nomes) usando o libmain.so da versão Android
(ARMv7 Thumb, 1.2.92), que exporta ~6000 funções C++ com nome.

Uso: python re/tools/match_android_symbols.py Breach.exe libmain.so re/evidencias/
Saída: <saida>/nomes_android.csv e <saida>/nomes_android_resumo.md

Três sinais, do mais forte ao mais fraco:
  1. lua    — registros luabind já inventariados (classe + nome Lua + assinatura) ↔ símbolo C++ de mesmo
              nome e mesmos parâmetros;
  2. texto  — strings literais citadas por exatamente uma função em cada binário (votação, par mútuo);
  3. vizinhos — chamadores e chamados já casados votam no nome (validação: 68 certos, 1 errado). Alinhar
              a ORDEM das chamadas não funciona (MSVC e clang inlinam funções diferentes: 0 de 72).
Nenhum arquivo do jogo é copiado; só nomes e endereços vão para a saída.
"""
import bisect, csv, re, struct, subprocess, sys
from collections import Counter, defaultdict
from pathlib import Path

import capstone
import pefile
from elftools.elf.elffile import ELFFile

MIN_STR = 4


# ------------------------------------------------------------------ ARM (libmain.so)
def load_arm(path):
    data = Path(path).read_bytes()
    elf = ELFFile(open(path, 'rb'))
    secs = {s.name: s for s in elf.iter_sections()}
    text = secs['.text']
    ro = secs['.rodata']
    ro_lo, ro_hi = ro['sh_addr'], ro['sh_addr'] + ro['sh_size']
    ro_off = ro['sh_offset'] - ro['sh_addr']

    def vread(addr, n):
        for s in elf.iter_sections():
            a, sz = s['sh_addr'], s['sh_size']
            if a and a <= addr < a + sz and s['sh_type'] != 'SHT_NOBITS':
                o = s['sh_offset'] + addr - a
                return data[o:o + n]
        return None

    def cstr(addr):
        if not (ro_lo <= addr < ro_hi):
            return None
        o = addr + ro_off
        end = data.find(b'\0', o, o + 400)
        if end < 0 or end - o < MIN_STR:
            return None
        s = data[o:end]
        if all(32 <= c < 127 or c in (9, 10) for c in s):
            return s.decode()
        return None

    funcs = {}
    dyn = secs['.dynsym']
    raw = []
    for sym in dyn.iter_symbols():
        if sym['st_info']['type'] == 'STT_FUNC' and sym['st_shndx'] != 'SHN_UNDEF' and sym['st_size'] > 0:
            raw.append((sym['st_value'], sym['st_size'], sym.name))
    names = [r[2] for r in raw]
    dem = subprocess.run(['c++filt'], input='\n'.join(names), capture_output=True, text=True).stdout.split('\n')
    for (val, size, mang), d in zip(raw, dem):
        start = val & ~1
        funcs.setdefault(start, {'start': start, 'size': size, 'thumb': bool(val & 1), 'mangled': mang, 'name': d})

    # PLT: chamadas entre funções exportadas passam pela PLT (símbolos preemptíveis). Entrada k da
    # .plt (cabeçalho de 20 bytes, entradas de 12) corresponde à relocação k da .rel.plt.
    by_mangled = {f['mangled']: f['start'] for f in funcs.values()}
    plt = secs['.plt']
    relplt = secs['.rel.plt']
    symtab = elf.get_section(relplt['sh_link'])
    plt_map = {}
    for k, rel in enumerate(relplt.iter_relocations()):
        sym = symtab.get_symbol(rel['r_info_sym'])
        tgt = by_mangled.get(sym.name)
        if tgt is not None:
            plt_map[plt['sh_addr'] + 20 + 12 * k] = tgt

    # vftables (_ZTV*): preenchidas por relocações na carga; aplica RELATIVE e ABS32 da .rel.dyn
    reloc_val = {}
    reldyn = secs['.rel.dyn']
    dsym = elf.get_section(reldyn['sh_link'])
    for rel in reldyn.iter_relocations():
        t = rel['r_info_type']
        if t == 23:  # R_ARM_RELATIVE: valor já está no lugar
            v = vread(rel['r_offset'], 4)
            if v:
                reloc_val[rel['r_offset']] = struct.unpack('<I', v)[0]
        elif t in (2, 21):  # R_ARM_ABS32 / R_ARM_GLOB_DAT
            sym = dsym.get_symbol(rel['r_info_sym'])
            if sym['st_value']:
                reloc_val[rel['r_offset']] = sym['st_value']
    vtables = {}
    for sym in dyn.iter_symbols():
        if sym.name.startswith('_ZTV') and sym['st_info']['type'] == 'STT_OBJECT' and sym['st_size'] >= 12:
            ents = []
            for k in range(2, sym['st_size'] // 4):
                ent = reloc_val.get(sym['st_value'] + 4 * k)
                if ent is None or (ent & ~1) not in funcs and not (text['sh_addr'] <= (ent & ~1) < text['sh_addr'] + text['sh_size']):
                    break   # fim da tabela primária
                ents.append(ent & ~1)
            vtables[sym.name[4:]] = ents

    md_t = capstone.Cs(capstone.CS_ARCH_ARM, capstone.CS_MODE_THUMB)
    md_a = capstone.Cs(capstone.CS_ARCH_ARM, capstone.CS_MODE_ARM)
    for f in funcs.values():
        code = vread(f['start'], f['size']) or b''
        md = md_t if f['thumb'] else md_a
        lit = {}
        strs, calls = [], []
        for ins in md.disasm(code, f['start']):
            m, op = ins.mnemonic, ins.op_str
            if m.startswith('ldr') and '[pc' in op:
                mm = re.match(r'(r\d+|ip|lr), \[pc, #(-?0x[0-9a-f]+|-?\d+)\]', op)
                if mm:
                    disp = int(mm.group(2), 0)
                    pc = (ins.address + (4 if f['thumb'] else 8)) & ~3
                    v = vread(pc + disp, 4)
                    if v:
                        lit[mm.group(1)] = struct.unpack('<I', v)[0]
                continue
            mm = re.match(r'(r\d+|ip|lr), pc$', op)
            if m.startswith('add') and mm and mm.group(1) in lit:
                addr = (lit.pop(mm.group(1)) + ins.address + (4 if f['thumb'] else 8)) & 0xffffffff
                s = cstr(addr)
                if s:
                    strs.append(s)
                continue
            if m in ('bl', 'blx') and op.startswith('#'):
                t = int(op[1:], 0) & ~1
                calls.append(plt_map.get(t, t))
        f['strings'] = strs
        f['calls'] = calls
    return funcs, vtables


# ------------------------------------------------------------------ PC (Breach.exe)
def load_pc(path):
    pe = pefile.PE(path, fast_load=True)
    data = pe.__data__
    base = pe.OPTIONAL_HEADER.ImageBase
    secs = {s.Name.rstrip(b'\0').decode(): s for s in pe.sections}
    t = secs['.text']
    t_va, t_off, t_sz = base + t.VirtualAddress, t.PointerToRawData, t.SizeOfRawData
    code = data[t_off:t_off + t_sz]

    strings = {}
    for name in ('.rdata', '.data'):
        s = secs.get(name)
        if not s:
            continue
        va0, o0, sz = base + s.VirtualAddress, s.PointerToRawData, s.SizeOfRawData
        blob = data[o0:o0 + sz]
        for m in re.finditer(rb'[\x20-\x7e\t\n]{%d,400}\x00' % MIN_STR, blob):
            if m.start() == 0 or blob[m.start() - 1] == 0:
                strings[va0 + m.start()] = m.group()[:-1].decode()

    # varredura linear (capstone): limites reais de instrução. Inícios de função = alvos de CALL
    # direto + primeira instrução depois de uma sequência de int3 (preenchimento do MSVC) ou de ret.
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    md.skipdata = True
    starts = set()
    calls_at = []          # (endereço da instrução, alvo)
    str_refs = []          # (endereço da instrução, string)
    prev_pad = True
    for addr, size, mn, ops in md.disasm_lite(code, t_va):
        if mn == 'int3' or mn == '.byte':
            prev_pad = True
            continue
        if prev_pad:
            starts.add(addr)
            prev_pad = False
        if mn == 'call' and ops.startswith('0x'):
            tgt = int(ops, 16)
            if t_va <= tgt < t_va + t_sz:
                starts.add(tgt)
                calls_at.append((addr, tgt))
        if size >= 5:
            o = addr - t_va
            for k in range(1, size - 3):
                v = struct.unpack_from('<I', code, o + k)[0]
                if v in strings:
                    str_refs.append((addr, strings[v]))
                    break
    starts = sorted(starts)

    def owner(va):
        k = bisect.bisect_right(starts, va) - 1
        return starts[k] if k >= 0 else None

    fn_strings = defaultdict(list)
    for a, st in str_refs:
        fn_strings[owner(a)].append(st)
    fn_calls = defaultdict(list)
    for a, tgt in calls_at:
        fn_calls[owner(a)].append(tgt)
    # vftables por RTTI (Complete Object Locator → TypeDescriptor ".?AV<nome>@@"), só offset 0
    rd = secs['.rdata']
    rd_va, rd_off, rd_sz = base + rd.VirtualAddress, rd.PointerToRawData, rd.SizeOfRawData
    def u32(va):
        o = va - base
        for sec in pe.sections:
            if sec.VirtualAddress <= o < sec.VirtualAddress + sec.SizeOfRawData:
                return struct.unpack_from('<I', data, sec.PointerToRawData + o - sec.VirtualAddress)[0]
        return None
    tds = {}
    for m in re.finditer(rb'\.\?AV([A-Za-z_0-9]+)@@\x00', data):
        o = m.start() - 8
        for sec in pe.sections:
            if sec.PointerToRawData <= o < sec.PointerToRawData + sec.SizeOfRawData:
                tds[base + sec.VirtualAddress + o - sec.PointerToRawData] = m.group(1).decode()
    cols = {}
    for o in range(rd_off, rd_off + rd_sz - 20, 4):
        if struct.unpack_from('<I', data, o)[0] == 0 and struct.unpack_from('<I', data, o + 4)[0] == 0:
            td = struct.unpack_from('<I', data, o + 12)[0]
            if td in tds:
                cols[rd_va + o - rd_off] = tds[td]
    pc_vt = {}
    for o in range(rd_off, rd_off + rd_sz - 4, 4):
        v = struct.unpack_from('<I', data, o)[0]
        if v in cols and cols[v] not in pc_vt:
            ents, va = [], rd_va + o - rd_off + 4
            while True:
                e = u32(va)
                if e is None or not (t_va <= e < t_va + t_sz) or (va != rd_va + o - rd_off + 4 and u32(va - 0) in cols):
                    break
                ents.append(e)
                va += 4
                if u32(va) in cols:
                    break
            pc_vt[cols[v]] = ents
    return starts, fn_strings, fn_calls, pc_vt


# ------------------------------------------------------------------ casamento
def norm_params(sig):
    s = sig
    for a, b in (('class ', ''), ('struct ', ''), ('enum ', ''), ('std::basic_string<char,std::char_traits<char>,std::allocator<char> >', 'string'),
                 ('std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >', 'string'),
                 (' ', '')):
        s = s.replace(a, b)
    return s


def main():
    exe, so, outdir = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    arm, arm_vt = load_arm(so)
    starts, pc_str, pc_calls, pc_vt = load_pc(exe)
    match = {}  # pc_va -> (arm_start, método, evidência)

    # 1. registros luabind
    by_class_name = defaultdict(list)
    for f in arm.values():
        m = re.match(r'(?:non-virtual thunk to )?([A-Za-z_][\w:]*)::(~?\w+)\((.*)\)( const)?$', f['name'])
        if m and not f['name'].startswith('non-virtual'):
            by_class_name[(m.group(1), m.group(2))].append((f, norm_params(m.group(3))))
    api = Path(outdir) / 'lua_api_v2_all.csv'
    for r in csv.DictReader(open(api, encoding='utf-8')):
        if r['kind'] != 'method' or not r['resolved_target_va']:
            continue
        sig = r['cpp_signature_demangled']
        mm = re.search(r'\(__thiscall (\w+)::\*\)\((.*?)\)', sig)
        if not mm:
            continue
        cls, params = mm.group(1), norm_params(mm.group(2)).replace('void', '')
        cands = by_class_name.get((cls, r['name']), [])
        if len(cands) > 1:
            cands = [c for c in cands if c[1] == params] or cands
        how = 'lua'
        if not cands:   # nome C++ diferente do nome Lua (ex.: GetReachable → GetReachablePointList)
            # só prefixo (ou "Set"+nome); nada para consultas "Is*", que costumam ter irmãs (IsFire/IsFireImmune)
            ok = lambda n: not r['name'].startswith('Is') and (n.startswith(r['name']) or n == 'Set' + r['name'])
            cands = [c for (k, n), lst in by_class_name.items() if k == cls and ok(n) for c in lst if c[1] == params]
            how = 'lua~'
        if len(cands) == 1:
            pc = int(r['resolved_target_va'], 16)
            match.setdefault(pc, (cands[0][0]['start'], how, f"{r['lua_class']}:{r['name']}"))

    # 1b. vftables: mesma classe, mesma posição. O destrutor ocupa 1 entrada no MSVC e 2 no Itanium.
    vcheck = Counter()
    vt_pairs = {}
    def mangled_class(n):
        return f'{len(n)}{n}'
    for cls, pents in pc_vt.items():
        aents = arm_vt.get(mangled_class(cls))
        if not aents:
            continue
        aligned, k = [], 0
        while k < len(aents):
            nm = arm[aents[k]]['name'] if aents[k] in arm else ''
            nxt = arm[aents[k + 1]]['name'] if k + 1 < len(aents) and aents[k + 1] in arm else ''
            aligned.append(aents[k])
            k += 2 if ('~' in nm and '~' in nxt) else 1
        if len(aligned) != len(pents):
            continue
        for pv, av in zip(pents, aligned):
            if av not in arm or 'pure_virtual' in arm[av]['name']:
                continue
            if pv in match:
                vcheck[match[pv][0] == av] += 1
            vt_pairs.setdefault(pv, set()).add(av)
    for pv, avs in vt_pairs.items():
        if len(avs) == 1 and pv not in match:
            match[pv] = (next(iter(avs)), 'vtable', 'mesma posição na vftable')

    # 2. strings únicas
    arm_by_s, pc_by_s = defaultdict(set), defaultdict(set)
    for f in arm.values():
        for s in f['strings']:
            arm_by_s[s].add(f['start'])
    for p, ss in pc_str.items():
        for s in ss:
            pc_by_s[s].add(p)
    votes = defaultdict(Counter)
    rvotes = defaultdict(Counter)
    for s, a in arm_by_s.items():
        p = pc_by_s.get(s)
        if p and len(a) == 1 and len(p) == 1:
            pa, aa = next(iter(p)), next(iter(a))
            votes[pa][aa] += 1
            rvotes[aa][pa] += 1
    tcheck = Counter()
    for p, c in votes.items():
        (a, n), = c.most_common(1)
        if rvotes[a].most_common(1)[0][0] != p:
            continue
        if p in match:
            tcheck[match[p][0] == a] += 1

        else:
            match[p] = (a, 'texto', f'{n} string(s)')

    # 3. vizinhos no grafo de chamadas: para uma função do PC sem nome, cada chamador e cada chamado já
    #    casados "votam" nas funções do Android ligadas a eles da mesma forma. Aceita com >= 3 votos, o
    #    dobro do segundo colocado e metade dos vizinhos casados concordando. Validação deixando cada
    #    casamento de fora (lua/texto): 68 certos, 1 errado. Repetido até estabilizar.
    pc_out = {p: set(v) for p, v in pc_calls.items()}
    pc_in = defaultdict(set)
    for p, v in pc_out.items():
        for c in v:
            pc_in[c].add(p)
    a_out = {a: set(f['calls']) & set(arm) for a, f in arm.items()}
    a_in = defaultdict(set)
    for a, v in a_out.items():
        for c in v:
            a_in[c].add(a)

    def predict(x, M, used):
        labels = [('in', M[p][0]) for p in pc_in.get(x, ()) if p in M and p != x]
        labels += [('out', M[c][0]) for c in pc_out.get(x, ()) if c in M and c != x]
        if len(labels) < 3:
            return None
        sc = Counter()
        for d, a in labels:
            cands = a_out.get(a, ()) if d == 'in' else a_in.get(a, ())
            if len(cands) > 60:
                continue
            for y in cands:
                if y not in used:
                    sc[y] += 1
        if not sc:
            return None
        top = sc.most_common(2)
        y, n = top[0]
        second = top[1][1] if len(top) > 1 else 0
        if n >= 3 and n >= 2 * max(second, 0.5) and n >= 0.5 * len(labels):
            return y, n, len(labels)
        return None

    used = {a for a, _, _ in match.values()}
    for _round in range(10):
        new_pairs = {}
        for x in set(pc_in) | set(pc_out):
            if x in match:
                continue
            r = predict(x, match, used)
            if r:
                new_pairs[x] = r
        # cada função do Android só pode ir para uma do PC
        claimed = Counter(y for y, _, _ in new_pairs.values())
        added = 0
        for x, (y, n, tot) in new_pairs.items():
            if claimed[y] == 1 and y not in used:
                match[x] = (y, 'vizinhos', f'{n} de {tot} vizinhos')
                used.add(y)
                added += 1
        if not added:
            break

    rows = []
    for p in sorted(match):
        a, how, ev = match[p]
        f = arm.get(a)
        conf = 'alta' if how in ('lua', 'vtable') or (how == 'texto' and not ev.startswith('1 ')) else 'media'
        rows.append({'pc_va': f'0x{p:08X}', 'android_va': f'0x{a:08X}', 'nome': f['name'] if f else '?',
                     'metodo': how, 'confianca': conf, 'evidencia': ev})
    with open(outdir / 'nomes_android.csv', 'w', newline='', encoding='utf-8') as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader(); w.writerows(rows)
    cnt = Counter(r['metodo'] for r in rows)
    print('confiança:', dict(Counter(r['confianca'] for r in rows)))
    print(f"funções com nome no Android: {len(arm)} | casadas com o PC: {len(rows)} {dict(cnt)}")
    print(f"validação da vftable contra lua: concorda {vcheck[True]}, discorda {vcheck[False]}")
    print(f"validação do texto contra lua: concorda {tcheck[True]}, discorda {tcheck[False]}")


if __name__ == '__main__':
    main()
