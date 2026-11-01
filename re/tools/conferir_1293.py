#!/usr/bin/env python3
"""Confere na 1.2.93 os fatos de que as regras em Rust dependem (só leitura do EXE).

Não precisa do EXE 1.2.88: cada fato é uma leitura direta nos endereços novos ou uma prova cruzada
entre sinais independentes (slot de vftable x tabela de migração). Falha (exit != 0) em qualquer
divergência.

Uso: python re/tools/conferir_1293.py <Breach.exe 1.2.93>
"""
import csv, hashlib, re, struct, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from map_lua_api_v2 import EXE_SHA256_1293, Analyzer  # noqa: E402

TABLE = {r['va_1288']: r['va_1293'] for r in csv.DictReader(
    (HERE.parent / 'evidencias' / 'v1.2.93' / 'migracao_enderecos.csv').open(encoding='utf-8'))}


def new(old):
    return int(TABLE[f'0x{old:08X}'], 16)


def main():
    data = Path(sys.argv[1]).read_bytes()
    assert hashlib.sha256(data).hexdigest() == EXE_SHA256_1293, 'EXE diferente'
    an = Analyzer(data)
    pe = an.pe
    n = 0

    def check(desc, ok):
        nonlocal n
        n += 1
        print(('ok    ' if ok else 'FALHA ') + desc)
        if not ok:
            raise SystemExit(1)

    def body(va, size=0x400):
        return list(an.md.disasm(data[pe.off(va):pe.off(va) + size], va))

    def text(va, size=0x400):
        return '\n'.join(f'{i.mnemonic} {i.op_str}' for i in body(va, size))

    def f32(va):
        return struct.unpack_from('<f', data, pe.off(va))[0]

    def strings(va, size):
        found = []
        for t in re.findall(r'push (0x[0-9a-f]{6})\b', text(va, size)):
            s = pe.cstr(int(t, 16))
            if s:
                found.append(s)
        return found

    def vt(typ, offset):
        return an.vtables[typ][offset]

    # vftables x tabela de migração
    board0, board12 = vt('.?AVBoard@@', 0), vt('.?AVBoard@@', 12)
    for desc, table_va, slot, old in (('GetSize', board0, 0x04, 0x0055DC10), ('IsEnterable', board0, 0x14, 0x00561300),
                                      ('GetEnterCost', board0, 0x18, 0x005612A0), ('GetLeaveCost', board0, 0x1C, 0x00561280),
                                      ('executor', board12, 0x0C, 0x005601C0), ('IsTerrain', board12, 0x7C, 0x0055F0D0),
                                      ('GetSimplePath', board12, 0x80, 0x005731F0)):
        check(f'Board: slot {slot:#x} = {desc} na tabela de migração', pe.u32(table_va + slot) == new(old))
    active = vt('.?AVActiveLua@@', 0)
    for slot, old in ((4, 0x00675C70), (8, 0x00675D00), (12, 0x00675E10)):
        check(f'ActiveLua: slot {slot} na tabela de migração', pe.u32(active + slot) == new(old))

    # ciclo de vida (re/evidencias/ciclo_de_vida.md)
    check('ActiveLua slot 1 chama SetPawn/SetBoard/SetGame', strings(new(0x00675C70), 0x90) == ['SetPawn', 'SetBoard', 'SetGame'])
    check('ActiveLua slot 2 lê scripts/scripts.lua e GetScripts',
          {'scripts/scripts.lua', 'GetScripts'} <= set(strings(new(0x00675D00), 0x110)))

    # tempos do executor
    push = new(0x0062CC00)
    check('Pawn::Push arma 0,4 s em +0x968', 'mov dword ptr [esi + 0x968], 0x3ecccccd' in text(push, 0x1C0))
    check('Pawn::Push grava a direção em +0x978', 'mov dword ptr [esi + 0x978], edi' in text(push, 0x1C0))
    check('Pawn::Push: FLIP testa +0x91F, os outros testam +0x91E',
          text(push, 0x80).index('0x91f') < text(push, 0x80).index('0x91e'))
    check('Pawn::Melee arma 0,15 s', '0x3e19999a' in text(new(0x00632B40), 0x180))

    # definição do tipo em Pawn::OnInit
    oninit = text(0x00629700, 0x1200)
    check('OnInit: +0x91E = Pushable', pe.cstr(0x00832374) == 'Pushable' and '0x91e' in oninit)
    check('OnInit: +0x91F = não IgnoreFlip', pe.cstr(0x00832368) == 'IgnoreFlip' and 'sete al' in oninit and '0x91f' in oninit)

    # mapa de missão (re/evidencias/v1.2.93/rotinas_mapa.md)
    check('BoardSpace nasce com Health(4)', 'push 4' in text(0x00599DC0, 0x17) and 'call 0x4e0ce0' in text(0x00599DC0, 0x17))
    check('LoadScript: health_max tem padrão 2', pe.cstr(0x0082B4C0) == 'health_max' and 'push 2' in text(0x00563080, 0x21))
    check('LoadScript aplica lava com SetTerrain(3)', 'push 3' in text(0x00562B1F, 0xD) and 'call 0x5a1b10' in text(0x00562B1F, 0xD))
    check('GetZone: zona ausente testa "enemy"', pe.cstr(0x0082B904) == 'enemy' and '0x82b904' in text(0x0056DF66, 0x18))

    # unidades (re/evidencias/v1.2.93/rotinas_unidades.md)
    check('contador de id começa em 4', struct.unpack_from('<i', data, pe.off(0x0088FD64))[0] == 4)
    check('Pawn::Pawn incrementa o id', 'mov dword ptr [esi + 0x9a4], eax' in text(0x00628DA7, 0x12))
    empty = text(0x00642670, 0xB0)
    check('CreateEmptyPawn: tipo ausente vira PunchMech', pe.cstr(0x0082B520) == 'PunchMech' and '0x82b520' in empty)
    check('CreateEmptyPawn: time 2 usa GetDefaultTeam', 'cmp esi, 2' in empty and pe.cstr(0x00833360) == 'GetDefaultTeam')
    check('CreatePawn(string) passa time 2', 'push 2' in text(0x006425E0, 0x30))
    add = text(0x0056E830, 0x3D0)
    check('AddPawn: três passadas e Filler_Pawn', 'cmp edx, 3' in add and 'cmp edx, 2' in add and pe.cstr(0x00827F44) == 'Filler_Pawn' and '0x827f44' in add)
    check('getter de inteiro monta "Get"', pe.cstr(0x0081CE68) == 'Get' and '0x81ce68' in text(0x004492B0, 0x50))

    check('Pawn::Pawn chama rand depois de OnInit', 'call 0x7855e3' in text(0x00628D95, 0xC) and 'call 0x629700' in text(0x00628D95, 0xC))

    # surgimento e Game (re/evidencias/v1.2.93/rotinas_unidades.md)
    sector = text(0x0060D9A0, 0x33)
    check('GetSector: +0x5238 + 1 em 1..4', '0x5238' in sector and 'inc eax' in sector and 'mov eax, 4' in sector)
    check('GetTeamTurn sem BoardPlayer dá 2', 'mov eax, 2' in text(0x0060D971, 0x6))
    check('IsIslandUnlocked compara com o perfil', 'setle al' in text(0x0060A110, 0x3B))
    check('SpawnPawn(Pawn*) usa (-1,-1)', text(0x005690C0, 0x10).count('push -1') == 2)
    space = text(0x00572A00, 0x470)
    check('GetAvailableSpace: rand, zona enemy e time 9', 'call 0x7855e3' in space and '0x82b904' in space and 'push 9' in space)
    avail = text(0x00572E70, 0x230)
    check('teste de casa: item, cápsula, tipos de bloqueio', all(k in avail for k in ('0x2b04', '0x2ae4', '0x7458')))
    check('fila de surgimento grava três listas', all(k in text(0x005693D7, 0x30) for k in ('0x2d44', '0x2d50', '0x2d5c')))
    check('casa bloqueada: mech com Passive_Burrows', pe.cstr(0x0082B634) == 'Passive_Burrows' and 'push 4' in text(0x00566786, 0x4))

    # valores
    check('A*: 1,01f', abs(f32(new(0x00837E1C)) - 1.01) < 1e-6)
    check('Normalize: 255,0f', f32(new(0x00838028)) == 255.0)
    rand = text(new(0x0078478E), 0x20)
    check('rand: LCG 214013/2531011, >>16 & 0x7FFF', all(k in rand for k in ('0x343fd', '0x269ec3', 'shr ecx, 0x10', '0x7fff')))
    check('srand grava a semente', 'mov dword ptr [eax + 0x18], ecx' in text(new(0x007847AF), 0x12))
    check('strings de Point', [pe.cstr(new(a)) for a in (0x00822174, 0x0081CEBC, 0x00822170)] == ['Point( ', ', ', ' )'])
    check('StyleInfo() usa "classic"', pe.cstr(0x00822020) == 'classic' and '0x822020' in text(new(0x006D6E6E), 0x30))

    # chamadas que ligam rotinas migradas
    check('Point::__mul chama roundf', f'call {new(0x00780857):#x}' in text(new(0x00451F60), 0x90))
    check('Board::PushDamage chama SpaceDamage(int)', f'call {new(0x00598AB0):#x}' in text(new(0x0055F680), 0x100))
    for d, lbl in (((0x008CF9AC, 0x004CDE60), 'Dijkstra'), ((0x008CF9AC, 0x004CE630), 'A*'), ((0x008C6800, 0x004B2340), 'GetDirection'),
                   ((0x008D0DE0, 0x0062CC00), 'Pawn::Push'), ((0x008CF84C, 0x00629540), 'IsCorpse'), ((0x008D0E94, 0x0065AEB0), 'IsPassiveSkill'),
                   ((0x008D0E48, 0x0065AEB0), 'IsPassiveSkill'), ((0x0088F10C, 0x004B2560), 'ScreenSizeX'), ((0x0088F114, 0x004B2560), 'ScreenSizeX'),
                   ((0x0088EB4C, 0x004B2560), 'ScreenSizeX')):
        check(f'{lbl} lê {TABLE[f"0x{d[0]:08X}"]}', f'{new(d[0]):#x}' in text(new(d[1]), 0x1000))
    print(f'PASS: {n} fatos conferidos na 1.2.93.')


if __name__ == '__main__':
    main()
