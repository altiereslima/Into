#!/usr/bin/env python3
"""Gera crates/itb-lua/src/api_manifest.rs a partir do inventario extraido do EXE.

Entrada: re/evidencias/lua_api_v2_all.csv, lua_api_v2_classes.csv, lua_global_values.csv
Saida:   crates/itb-lua/src/api_manifest.rs (codigo Rust, versionado no repositorio)

O manifesto e a ESPECIFICACAO da API que o host Rust precisa oferecer aos scripts originais.
Uso: python re/tools/gen_api_manifest.py
"""
import csv
from collections import OrderedDict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EV = ROOT / 're' / 'evidencias'
OUT = ROOT / 'crates' / 'itb-lua' / 'src' / 'api_manifest.rs'

# Instancias globais criadas pelo motor (nome em _G -> classe Lua). PAWN_FACTORY vem de
# lua_global_values.csv (tipo 'object'); Board/Game sao atribuidos em tempo de missao.
INSTANCES = [('Board', 'Board'), ('Game', 'GameMap'), ('PAWN_FACTORY', 'PawnFactory')]


def rs(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'


def short_sig(sig):
    for a, b in (('class ', ''), ('struct ', ''), ('__thiscall ', ''), ('__cdecl ', ''),
                 ('std::basic_string<char, std::char_traits<char>, std::allocator<char>>', 'string'),
                 ('luabind::detail::', ''), (', null_type>', '>')):
        sig = sig.replace(a, b)
    return sig


def main():
    rows = list(csv.DictReader((EV / 'lua_api_v2_all.csv').open(encoding='utf-8')))
    classes = [r['lua_class'] for r in csv.DictReader((EV / 'lua_api_v2_classes.csv').open(encoding='utf-8'))]
    methods = OrderedDict((c, OrderedDict()) for c in classes)
    props = OrderedDict((c, []) for c in classes)
    ctors = OrderedDict((c, 0) for c in classes)
    globals_ = OrderedDict()
    for r in rows:
        k, c, n = r['kind'], r['lua_class'], r['name']
        if k == 'method':
            methods[c].setdefault(n, []).append((r['resolved_target_va'], short_sig(r['cpp_signature_demangled'])))
        elif k == 'property':
            if n not in props[c]:
                props[c].append(n)
        elif k == 'constructor':
            ctors[c] += 1
        elif k == 'global_function':
            globals_.setdefault(n, []).append((r['resolved_target_va'], short_sig(r['cpp_signature_demangled'])))
    consts = OrderedDict()
    for r in csv.DictReader((EV / 'lua_global_values.csv').open(encoding='utf-8')):
        n, t, v = r['name'], r['value_type'], r['value']
        if t == 'integer':
            val = f'Const::Int({int(v)})'
        elif t == 'boolean':
            val = f'Const::Bool({v})'
        elif t == 'Point':
            x, y = v.split(';'); val = f'Const::Point({x}, {y})'
        elif t == 'GL_Color':
            parts = [('f32::MAX' if float(p) > 3e38 else f'{float(p)!r}_f32') for p in v.split(';')]
            val = 'Const::Color(' + ', '.join(parts) + ')'
        else:
            continue
        if n in consts and consts[n] != val:
            raise SystemExit(f'constante {n} com valores divergentes: {consts[n]} vs {val}')
        consts[n] = val

    L = ['// @generated por re/tools/gen_api_manifest.py — NAO EDITAR A MAO.',
         '// Fonte: inventario do Breach.exe v1.2.88 (SHA256 4bd9e4cc...84b0).',
         '#![allow(clippy::all)]', '',
         '/// Sobrecarga nativa: (VA do corpo resolvido, assinatura C++ simplificada).',
         'pub type Overload = (&\'static str, &\'static str);', '',
         'pub struct MethodDef { pub name: &\'static str, pub overloads: &\'static [Overload] }',
         'pub struct ClassDef {', '    pub name: &\'static str,', '    pub methods: &\'static [MethodDef],',
         '    pub properties: &\'static [&\'static str],', '    pub constructors: usize,', '}',
         'pub struct GlobalFnDef { pub name: &\'static str, pub overloads: &\'static [Overload] }', '',
         '#[derive(Clone, Copy, Debug, PartialEq)]',
         'pub enum Const { Int(i64), Bool(bool), Point(i32, i32), Color(f32, f32, f32, f32) }', '',
         'pub static CLASSES: &[ClassDef] = &[']
    for c in classes:
        L.append(f'    ClassDef {{ name: {rs(c)}, constructors: {ctors[c]}, properties: &[{", ".join(rs(p) for p in props[c])}], methods: &[')
        for m, ovs in methods[c].items():
            L.append(f'        MethodDef {{ name: {rs(m)}, overloads: &[{", ".join(f"({rs(a)}, {rs(b)})" for a, b in ovs)}] }},')
        L.append('    ] },')
    L.append('];')
    L.append('')
    L.append('pub static GLOBAL_FUNCTIONS: &[GlobalFnDef] = &[')
    for g, ovs in globals_.items():
        L.append(f'    GlobalFnDef {{ name: {rs(g)}, overloads: &[{", ".join(f"({rs(a)}, {rs(b)})" for a, b in ovs)}] }},')
    L.append('];')
    L.append('')
    L.append('pub static CONSTANTS: &[(&str, Const)] = &[')
    for n, v in consts.items():
        L.append(f'    ({rs(n)}, {v}),')
    L.append('];')
    L.append('')
    L.append('/// Instancias globais do motor: (nome em _G, classe Lua).')
    L.append('pub static INSTANCES: &[(&str, &str)] = &[' + ', '.join(f'({rs(a)}, {rs(b)})' for a, b in INSTANCES) + '];')
    OUT.write_text('\n'.join(L) + '\n', encoding='utf-8')
    nm = sum(len(v) for v in methods.values())
    print(f'{OUT.relative_to(ROOT)}: {len(classes)} classes, {nm} metodos (nomes), {sum(len(v) for v in props.values())} '
          f'propriedades, {len(globals_)} funcoes globais (nomes), {len(consts)} constantes')


if __name__ == '__main__':
    main()
