#!/usr/bin/env python3
"""Desmonta o corpo dos construtores Lua (luabind `construct<T, auto_ptr<T>, vectorN<...>>`).

Para cada classe pedida, acha a vftable RTTI do function_object_impl<construct<...>>, segue as
chamadas a partir do slot de invocacao ate o `operator new` (0x00753C6F) e imprime as instrucoes
seguintes (onde os campos do objeto novo sao preenchidos ou o ctor C++ e chamado).

Uso: python re/tools/dump_ctor_bodies.py Breach.exe Rect2D [Classe ...]  > saida.txt
"""
import re, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
import map_lua_api_v2 as m

NEW = 0x00753C6F


def main():
    data = Path(sys.argv[1]).read_bytes()
    an = m.Analyzer(data)
    for cls in sys.argv[2:]:
        for name, offs in sorted(an.vtables.items()):
            mm = re.search(r'construct@[UV]' + re.escape(cls) + r'@@.*?vector(\d)@X(.*?)@mpl@boost', name)
            if not mm or 0 not in offs:
                continue
            print(f'===== {cls} vector{mm.group(1)} args={mm.group(2)}')
            seen = set()

            def scan(f, depth):
                if f in seen or depth > 3 or not an.pe.in_text(f):
                    return False
                seen.add(f)
                o = an.pe.off(f)
                ins = list(an.md.disasm(data[o:o + 0x600], f))
                for i, x in enumerate(ins):
                    if x.mnemonic == 'ret':
                        return False
                    if x.mnemonic == 'call' and x.op_str == hex(NEW):
                        for y in ins[i - 4:i + 28]:
                            print(f'  {y.address:08X}: {y.mnemonic} {y.op_str}')
                        return True
                    if x.mnemonic == 'call' and x.op_str.startswith('0x'):
                        if scan(int(x.op_str, 16), depth + 1):
                            return True
                return False
            scan(an.pe.u32(offs[0] + 4), 0)


if __name__ == '__main__':
    main()
