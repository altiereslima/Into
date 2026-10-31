#!/usr/bin/env python3
"""QA da Etapa 05: reproduz o inventario v2 e prova que ele CONTEM as etapas anteriores.

Uso: python tools/check_inventory_v2.py /caminho/Breach.exe /caminho/Into.the.Breach.v1.2.88.zip
Falha (exit != 0) em qualquer regressao.
"""
import collections, csv, hashlib, re, sys, zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from map_lua_api_v2 import EXE_SHA256, Analyzer, cpp_class_of          # noqa: E402
from map_all_lua_bindings import build as build_v1                    # noqa: E402

EXPECTED = {'method': 343, 'property': 111, 'constructor': 58, 'global_function': 39}


def main():
    if len(sys.argv) < 3:
        raise SystemExit('Uso: check_inventory_v2.py Breach.exe JOGO.zip')
    data = Path(sys.argv[1]).read_bytes()
    assert hashlib.sha256(data).hexdigest() == EXE_SHA256, 'EXE diferente'
    an = Analyzer(data)
    rows, classes, missed, regs = an.build()
    kinds = collections.Counter(r['kind'] for r in rows)
    assert dict(kinds) == EXPECTED, kinds
    assert len(classes) == 32 and all(c['name'] for c in classes), 'classes'
    assert not missed, f'wrappers globais nao capturados: {missed}'
    # 1) Todo registro tem tipo RTTI coerente com a familia.
    for r in rows:
        fam = {'method': 'memfun_registration', 'property': 'property_registration',
               'constructor': 'constructor_registration', 'global_function': 'function_registration'}[r['kind']]
        assert fam in (r['rtti'] or ''), ('RTTI incoerente', r)
        if r['kind'] != 'constructor':
            assert r['name'] and re.fullmatch(r'[A-Za-z_][A-Za-z_0-9]*', r['name']), r
        if r['kind'] in ('method', 'global_function'):
            assert r['target'] and an.pe.in_text(r['target']), r
        if r['kind'] != 'global_function':
            assert r['lua_class'], ('registro sem classe', r)
    # 2) Cada classe Lua corresponde a UMA unica classe C++ (template do registro) e vice-versa.
    m = collections.defaultdict(set)
    for r in rows:
        if r['kind'] != 'global_function':
            m[r['lua_class']].add(cpp_class_of(r['rtti']))
    assert all(len(v) == 1 for v in m.values()), {k: v for k, v in m.items() if len(v) != 1}
    cpps = [next(iter(v)) for v in m.values()]
    assert len(cpps) == len(set(cpps)), 'duas classes Lua com a mesma classe C++'
    # 3) Todo thunk virtual resolvido pela vftable RTTI.
    unresolved = []
    for r in rows:
        if r['kind'] == 'method':
            kind, slot, res = an.resolve(r['target'], r['this_adjust'], cpp_class_of(r['rtti']))
            if res is None or not an.pe.in_text(res):
                unresolved.append((r['lua_class'], r['name']))
    assert not unresolved, unresolved
    # 4) Os 228 registros estritos da Etapa 04 sao subconjunto exato.
    v1, cand = build_v1(data)
    pairs = {(r['name'], r['target']) for r in rows if r['kind'] == 'method' and r['name_form'] == 'arg'}
    for r in v1:
        assert (r['name'], int(r['native_target_va'], 16)) in pairs, ('regressao Etapa 04', r)
    assert len(v1) == 228
    # 5) As 83 entradas Board da Etapa 03 continuam presentes.
    with (HERE.parent / 'evidencias' / 'board_bindings.csv').open(encoding='utf-8', newline='') as f:
        old = list(csv.DictReader(f))
    board = {(r['name'], r['target']) for r in rows if r['lua_class'] == 'Board'}
    assert len(old) == 83
    for r in old:
        assert (r['method'], int(r['native_target'], 16)) in board, ('regressao Etapa 03', r)
    # 6) Os 3 "candidatos" da Etapa 04 foram classificados como PROPRIEDADES.
    props = {(r['lua_class'], r['name']) for r in rows if r['kind'] == 'property'}
    assert {('PilotSkill', 'desc'), ('Cost', 'money'), ('PilotInfo', 'type')} <= props
    # 7) Cruzamento com scripts: 110 pares objeto/metodo, todos com classe correta.
    objmap = {'Board': 'Board', 'Game': 'GameMap', 'PAWN_FACTORY': 'PawnFactory'}
    by_class = collections.defaultdict(set)
    for r in rows:
        by_class[r['lua_class']].add(r['name'])
    call = re.compile(r'\b(Board|Game|PAWN_FACTORY)[\.:]([A-Za-z_][A-Za-z_0-9]*)\s*\(')
    seen = set()
    with zipfile.ZipFile(sys.argv[2]) as z:
        assert z.read('Breach.exe') == data, 'ZIP e EXE divergentes'
        for n in z.namelist():
            if n.startswith('scripts/') and n.endswith('.lua'):
                for line in z.read(n).decode('utf-8', 'replace').splitlines():
                    seen.update(call.findall(line.split('--', 1)[0]))
    miss = [(o, m) for o, m in seen if m not in by_class[objmap[o]]]
    assert len(seen) == 110 and not miss, (len(seen), miss)
    # 8) Valores globais (constantes) atribuidos pelo EXE.
    gv = an.find_global_values()
    kinds_g = collections.Counter(x['value_type'] for x in gv)
    assert kinds_g == {'integer': 164, 'GL_Color': 21, 'Point': 4, 'boolean': 2, 'object': 1, 'read_or_unknown': 5}, kinds_g
    assert {x['name'] for x in gv if x['value_type'] == 'read_or_unknown'} == {'ANIMS', 'Profile', 'zones', 'tags', 'PawnList'}
    g = {x['name']: x['value'] for x in gv if x['value_type'] == 'integer'}
    assert g['TEAM_NONE'] == 2 and g['PATH_PHASING'] == 9 and g['TERRAIN_HOLE'] == 9 and g['TEAM_ENEMY'] == 6
    assert {x['name']: x['value'] for x in gv if x['value_type'] == 'Point'}['VEC_UP'] == '0;-1'
    # 9) FECHAMENTO: todo tipo RTTI de registro luabind do EXE e usado; toda escrita de vftable
    #    desses tipos ocorre dentro de um registrador conhecido; toda chamada a registrador vira linha.
    import struct
    fams = ('memfun_registration', 'property_registration', 'constructor_registration', 'function_registration')
    allvt = {vt: n for n, offs in an.vtables.items() if any(f in n for f in fams) for vt in offs.values()}
    used = {r['vt'] for r in rows}
    assert len(allvt) == 300 and set(allvt) <= used, ('tipos de registro sem uso', len(allvt), set(allvt) - used)
    known = set(regs) | {r['registrar'] for r in rows}
    sites = 0
    for vt in allvt:
        for mm in re.finditer(rb'\xc7[\x00-\x07]' + re.escape(struct.pack('<I', vt)), data[an.o0:an.o1]):
            sites += 1
            assert an.fstart(an.pe.text[1] + mm.start()) in known, ('escrita de vftable fora de registrador', hex(vt))
    per = collections.Counter(r['registrar'] for r in rows)
    via_wrapper = {r['registrar'] for r in rows if r['name_form'] == 'global_name_arg'}
    for k in known - via_wrapper:
        assert len(an.calls.get(k, [])) == per[k], ('chamada de registrador sem linha', hex(k))
    print('PASS: SHA256; 32 classes; 343 metodos, 111 propriedades, 58 construtores, 39 funcoes globais, '
          'todos tipados por RTTI; 0 thunks virtuais sem resolucao; 228 registros da Etapa 04 e 83 da Etapa 03 '
          'contidos; desc/money/type = propriedades; 110/110 pares dos scripts casados com a classe; 164 constantes inteiras + 21 cores + 4 vetores + 2 booleanos; '
          f'fechamento RTTI: 300/300 tipos de registro usados, {sites} escritas de vftable dentro de registradores.')


if __name__ == '__main__':
    main()
