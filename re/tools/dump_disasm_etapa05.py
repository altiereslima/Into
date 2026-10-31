#!/usr/bin/env python3
"""Gera as desmontagens usadas como prova na Etapa 05 (somente leitura do EXE).
Uso: python tools/dump_disasm_etapa05.py /caminho/Breach.exe evidencias/etapa05_disasm
"""
import hashlib, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from map_lua_api_v2 import EXE_SHA256, PE
import capstone

ROUTINES = {
    'board_getpawn_point_0x570E00': 0x00570E00, 'board_getpawn_point_bool_0x567750': 0x00567750,
    'pawnholder_getpawn_id_0x567700': 0x00567700, 'board_ispawnalive_0x567840': 0x00567840,
    'board_getpawnspace_0x567970': 0x00567970, 'weapontarget_ispawnspace_point_0x56EE50': 0x0056EE50,
    'weapontarget_ispawnspace_point_bool_0x56EE70': 0x0056EE70, 'board_isblocked_0x55F030': 0x0055F030,
    'board_getpawnteam_0x568950': 0x00568950, 'tile_haspawn_0x59F050': 0x0059F050,
    'tile_getpawn_0x5AE490': 0x005AE490, 'tile_terrainblocks_0x5AE510': 0x005AE510,
    'pawn_isdead_0x629530': 0x00629530, 'pawn_iscorpse_0x629540': 0x00629540,
    'pawn_getspace_0x635FA0': 0x00635FA0, 'pawn_gethealth_0x4F6410': 0x004F6410,
    'pawn_isdamaged_0x638D50': 0x00638D50, 'pawn_getteam_0x639FC0': 0x00639FC0,
    'pawn_setteam_0x62F2D0': 0x0062F2D0, 'pawn_isplayer_0x62F840': 0x0062F840,
    'pawn_isenemy_0x62F870': 0x0062F870, 'pawn_getid_0x6370D0': 0x006370D0,
    'pawn_ismech_0x639040': 0x00639040, 'pawn_setmech_0x639050': 0x00639050,
}

def main():
    data = Path(sys.argv[1]).read_bytes()
    if hashlib.sha256(data).hexdigest() != EXE_SHA256:
        raise SystemExit('SHA256 inesperado')
    pe = PE(data); md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
    for name, va in ROUTINES.items():
        o = pe.off(va); lines = [f'; {name}  VA 0x{va:08X}  (Breach.exe v1.2.88, SHA256 {EXE_SHA256[:16]}...)']
        for i in md.disasm(data[o:o + 0x400], va):
            if i.mnemonic == 'int3':
                break
            lines.append(f'{i.address:08X}: {i.bytes.hex():<16} {i.mnemonic} {i.op_str}')
        # 0x5AE510 usa tabela de saltos logo apos o codigo: anexar os 9 alvos
        if va == 0x005AE510:
            lines = lines[:1] + [l for l in lines[1:] if int(l[:8], 16) < 0x005AE5AC]
            tbl = [int.from_bytes(data[pe.off(0x005AE5AC) + 4 * k:pe.off(0x005AE5AC) + 4 * k + 4], 'little') for k in range(9)]
            lines.append('; jump table 0x005AE5AC (terreno-1 = 0..8): ' + ', '.join(f'{k + 1}->0x{t:08X}' for k, t in enumerate(tbl)))
            lines.append('; string 0x0082D61C = "conveyor"')
        (out / f'{name}.txt').write_text('\n'.join(lines) + '\n', encoding='utf-8')
    print(f'{len(ROUTINES)} desmontagens em {out}')

if __name__ == '__main__':
    main()
