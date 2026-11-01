#!/usr/bin/env python3
"""Tabela de endereços 1.2.88 -> 1.2.93, feita sem o EXE 1.2.88 (só o que ficou em re/evidencias).

Uso (na raiz do repositório): python re/tools/migrar_enderecos.py <Breach.exe 1.2.93>
Antes: map_lua_api_v2.py e match_android_symbols.py com saída em re/evidencias/v1.2.93/.
Saída em re/evidencias/v1.2.93/: migracao_enderecos.csv, regioes_alteradas.csv, migracao_resumo.txt
e re/tools/rust_status.json (mesmo conteúdo de rust_status_1.2.88.json, com os endereços novos).

Sinais de pares (antigo, novo), conferidos entre si:
  inventario — mesmo registro luabind (tipo, classe, nome, ordem);
  android    — a mesma função do libmain.so casada nas duas versões;
  bytes      — prova da Etapa 05 achada na 1.2.93, com curingas nos campos de endereço;
  manual     — lido no assembly da 1.2.93, com a prova em pares_manuais.csv (tem precedência).
O código mantém a ordem, então o deslocamento só muda onde algo mudou de tamanho. Pares fora da
maior sequência crescente são descartados como casamento errado, assim como um par só do Android
cujo deslocamento difere dos dois vizinhos (o nome estava errado em uma das versões). Um endereço
sem par é interpolado quando as duas âncoras vizinhas têm o mesmo deslocamento e o resultado é
início de função.

Limite: duas âncoras com o mesmo deslocamento indicam tamanho total igual, não bytes iguais.
"""
import bisect, collections, csv, glob, json, re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import capstone  # noqa: E402
from map_lua_api_v2 import PE  # noqa: E402
from match_android_symbols import load_pc  # noqa: E402

EV = HERE.parent / 'evidencias'
NEW = EV / 'v1.2.93'
ROOT = HERE.parents[1]
LINE = re.compile(r'^([0-9A-Fa-f]{8}):\s+([0-9a-f]+)\s')


def read_csv(p):
    return list(csv.DictReader(open(p, encoding='utf-8')))


def inventory_pairs():
    """Mesma sequência de registros nas duas versões: casa pela ordem dentro de (tipo, classe, nome)."""
    def keyed(rows):
        seen, out = collections.Counter(), {}
        for r in rows:
            k = (r['kind'], r['lua_class'], r['name'])
            out[k + (seen[k],)] = r
            seen[k] += 1
        return out
    a, b = keyed(read_csv(EV / 'lua_api_v2_all.csv')), keyed(read_csv(NEW / 'lua_api_v2_all.csv'))
    if a.keys() != b.keys():
        print('registros diferentes:', sorted(a.keys() ^ b.keys())[:20])
    pairs, signature_diff = [], []
    for k in a.keys() & b.keys():
        if a[k]['cpp_registration_rtti'] != b[k]['cpp_registration_rtti']:
            signature_diff.append(k)
        for col in ('native_target_va', 'resolved_target_va', 'registrar_va', 'registration_vftable_va', 'name_string_va'):
            if a[k][col] and b[k][col]:
                pairs.append((int(a[k][col], 16), int(b[k][col], 16), 'inventario'))
    return pairs, signature_diff


def android_pairs():
    new = {r['android_va']: int(r['pc_va'], 16) for r in read_csv(NEW / 'nomes_android.csv')}
    return [(int(r['pc_va'], 16), new[r['android_va']], 'android')
            for r in read_csv(EV / 'nomes_android.csv') if r['android_va'] in new]


def bytes_pairs(pe, data):
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    md.detail = True
    _, base, size, o0 = pe.text
    out = []
    for f in sorted((EV / 'etapa05_disasm').glob('*.txt')):
        va, code = None, b''
        for ln in f.read_text(encoding='utf-8').splitlines():
            m = LINE.match(ln)
            if m:
                va = va if va is not None else int(m.group(1), 16)
                code += bytes.fromhex(m.group(2))
        wild = bytearray(len(code))
        for i in md.disasm(code, va):
            at = i.address - va
            if (i.mnemonic == 'call' or i.mnemonic.startswith('j')) and i.size >= 5 and i.imm_size:
                wild[at + i.imm_offset:at + i.imm_offset + i.imm_size] = b'\1' * i.imm_size
            if i.disp_size == 4 and (i.disp & 0xFFFFFFFF) >= 0x400000:
                wild[at + i.disp_offset:at + i.disp_offset + 4] = b'\1' * 4
            if i.imm_size == 4 and any(o.type == capstone.x86.X86_OP_IMM and (o.imm & 0xFFFFFFFF) >= 0x400000 for o in i.operands):
                wild[at + i.imm_offset:at + i.imm_offset + 4] = b'\1' * 4
        pat = b''.join(b'.' if w else re.escape(bytes([c])) for c, w in zip(code, wild))
        hits = [base + h.start() for h in re.finditer(pat, data[o0:o0 + size], re.S)]
        if len(hits) == 1:
            out.append((va, hits[0], 'bytes'))
    return out


def longest_increasing(items):
    """Maior sequência crescente em `novo`, com `items` ordenados por `antigo`."""
    tails, idx, prev = [], [], [None] * len(items)
    for i, (_, n, _m) in enumerate(items):
        k = bisect.bisect_left(tails, n)
        if k == len(tails):
            tails.append(n); idx.append(i)
        else:
            tails[k] = n; idx[k] = i
        prev[i] = idx[k - 1] if k else None
    keep, i = set(), idx[-1]
    while i is not None:
        keep.add(i)
        i = prev[i]
    return keep


def cited_addresses():
    cited = collections.Counter()
    files = glob.glob(str(ROOT / 'crates/**/*.rs'), recursive=True) + glob.glob(str(EV / '*.md')) + \
        glob.glob(str(ROOT / 'docs/*.md')) + [str(ROOT / 'CHECKPOINT.md')]
    for f in files:
        if f.endswith(('api_manifest.rs', 'nomes_android_resumo.md')):
            continue
        for m in re.finditer(r'0x00([4-9][0-9A-Fa-f]{5})\b', open(f, encoding='utf-8').read()):
            cited[int(m.group(1), 16)] += 1
    return cited


def main():
    data = Path(sys.argv[1]).read_bytes()
    pe = PE(data)
    starts, *_ = load_pc(sys.argv[1])
    starts = set(starts)
    inv, signature_diff = inventory_pairs()
    raw = inv + android_pairs() + bytes_pairs(pe, data)

    # conflitos: o mesmo endereço antigo levando a dois novos
    by_old = collections.defaultdict(lambda: collections.defaultdict(set))
    for o, n, m in raw:
        by_old[o][n].add(m)
    conflicts = {o: v for o, v in by_old.items() if len(v) > 1}
    for r in read_csv(NEW / 'pares_manuais.csv'):
        by_old[int(r['va_1288'], 16)] = {int(r['va_1293'], 16): {'manual'}}
    # em conflito, vence o inventário; o resto vai para a checagem de ordem
    items = []
    for o in sorted(by_old):
        v = by_old[o]
        n = next((k for k in v if 'inventario' in v[k]), None) or max(v, key=lambda k: len(v[k]))
        items.append((o, n, '+'.join(sorted(v[n]))))
    items.sort()
    code = [it for it in items if pe.in_text(it[0])]
    other = [it for it in items if not pe.in_text(it[0])]
    keep = longest_increasing(code)
    outliers = [it for i, it in enumerate(code) if i not in keep]
    code = [it for i, it in enumerate(code) if i in keep]
    spikes = []
    changed_any = True
    while changed_any:
        changed_any = False
        for i in range(1, len(code) - 1):
            d = code[i][1] - code[i][0]
            if code[i][2] == 'android' and d != code[i - 1][1] - code[i - 1][0] and d != code[i + 1][1] - code[i + 1][0]:
                spikes.append(code.pop(i))
                changed_any = True
                break
    anchors = code + other
    anchors.sort()
    olds = [a[0] for a in anchors]

    names = {int(r['pc_va'], 16): r['nome'] for r in read_csv(EV / 'nomes_android.csv')}
    for r in read_csv(EV / 'nomes_manual.csv'):
        names[int(r['pc_va'], 16)] = r['nome'].replace(';', ',')

    regions = []
    for (a, an, _), (b, bn, _) in zip(anchors, anchors[1:]):
        if pe.in_text(a) and pe.in_text(b) and an - a != bn - b:
            regions.append((a, b, an, bn, (bn - b) - (an - a)))

    cited = cited_addresses()
    rows, unresolved = [], []
    wanted = sorted(set(olds) | set(cited))
    exact = {a[0]: a for a in anchors}
    for o in wanted:
        k = bisect.bisect_right(olds, o) - 1
        prev_a = anchors[k] if k >= 0 else None
        next_a = anchors[bisect.bisect_right(olds, o)] if bisect.bisect_right(olds, o) < len(anchors) else None
        changed = bool(prev_a and next_a and prev_a[1] - prev_a[0] != next_a[1] - next_a[0])
        if o in exact:
            n, how = exact[o][1], exact[o][2]
            conf = 'alta'
        elif prev_a and next_a and not changed:
            n = o + prev_a[1] - prev_a[0]
            how = 'interpolado'
            conf = 'alta' if n in starts or not pe.in_text(n) else 'media'
        else:
            n, how, conf = None, 'sem_par', 'nenhuma'
            unresolved.append(o)
        rows.append({'va_1288': f'0x{o:08X}', 'va_1293': f'0x{n:08X}' if n else '',
                     'delta': f'0x{n - o:X}' if n else '', 'secao': 'text' if pe.in_text(o) else 'dados',
                     'nome': names.get(o, ''), 'metodo': how, 'confianca': conf, 'citacoes': cited.get(o, 0),
                     'regiao_alterada': int(changed)})

    with open(NEW / 'migracao_enderecos.csv', 'w', newline='', encoding='utf-8') as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys())); w.writeheader(); w.writerows(rows)
    with open(NEW / 'regioes_alteradas.csv', 'w', newline='', encoding='utf-8') as fh:
        w = csv.writer(fh)
        w.writerow(['ancora_1288', 'proxima_1288', 'ancora_1293', 'proxima_1293', 'cresceu_bytes', 'nome_ancora', 'nome_proxima'])
        for a, b, an, bn, g in regions:
            w.writerow([f'0x{a:08X}', f'0x{b:08X}', f'0x{an:08X}', f'0x{bn:08X}', g, names.get(a, ''), names.get(b, '')])

    # rust_status.json: os mesmos alvos, com os endereços novos
    table = {r['va_1288']: r['va_1293'] for r in rows if r['va_1293']}
    st = json.loads((HERE / 'rust_status_1.2.88.json').read_text(encoding='utf-8'))
    lost = []
    def tr(v):
        if v not in table:
            lost.append(v)
        return table.get(v, v)
    new_st = {'_doc': st['_doc'] + ' Enderecos da 1.2.93; a versao 1.2.88 esta em rust_status_1.2.88.json.',
              'done': [[a, b, tr(c)] for a, b, c in st['done']],
              'partial': [[a, b, tr(c)] for a, b, c in st['partial']],
              'trivial_accessors': [tr(v) for v in st['trivial_accessors']]}
    # o que já foi reconstruído direto na 1.2.93 não está no arquivo antigo: fica como está
    current = json.loads((HERE / 'rust_status.json').read_text(encoding='utf-8'))
    for key in ('done', 'partial', 'trivial_accessors'):
        new_st[key] += [e for e in current.get(key, []) if e not in new_st[key]]
    (HERE / 'rust_status.json').write_text(json.dumps(new_st, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

    cited_rows = [r for r in rows if r['citacoes']]
    L = [f'pares brutos: {len(raw)} | enderecos com par: {len(items)} | ancoras apos checar a ordem: {len(anchors)}',
         f'conflitos (mesmo antigo, dois novos): {len(conflicts)} | fora de ordem descartados: {len(outliers)} | picos descartados: {len(spikes)}',
         f'assinaturas luabind diferentes: {len(signature_diff)}',
         f'regioes em que o deslocamento muda: {len(regions)} | bytes somados: {sum(r[4] for r in regions)}',
         f'citados no projeto: {len(cited_rows)} | por metodo: {dict(collections.Counter(r["metodo"] for r in cited_rows))}',
         f'citados por confianca: {dict(collections.Counter(r["confianca"] for r in cited_rows))}',
         f'citados em regiao alterada: {sum(r["regiao_alterada"] for r in cited_rows)}',
         f'sem par: {len(unresolved)} ({sum(1 for o in unresolved if pe.in_text(o))} em .text)',
         f'rust_status sem endereco novo: {sorted(set(lost))}',
         'descartados: ' + ', '.join(f'0x{o:08X}->0x{n:08X}({m})' for o, n, m in outliers + spikes)]
    (NEW / 'migracao_resumo.txt').write_text('\n'.join(L) + '\n', encoding='utf-8', newline='\n')
    print('\n'.join(L))
    for o, v in list(conflicts.items())[:15]:
        print('conflito', hex(o), {hex(k): sorted(m) for k, m in v.items()})


if __name__ == '__main__':
    main()
