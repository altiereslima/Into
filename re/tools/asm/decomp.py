import sys, angr, logging
logging.getLogger('angr').setLevel(logging.ERROR)
logging.getLogger('cle').setLevel(logging.ERROR)
logging.getLogger('pyvex').setLevel(logging.ERROR)
import os; EXE=os.environ.get('ITB_EXE','Breach.exe')
p = angr.Project(EXE, auto_load_libs=False, load_options={'main_opts': {'base_addr': 0x400000}})
def decompile(addr, size=0x800):
    cfg = p.analyses.CFGFast(regions=[(addr, addr+size)], normalize=True, function_starts=[addr], force_complete_scan=False)
    f = cfg.kb.functions.get(addr)
    if f is None:
        return f'// sem função em {addr:#x}'
    d = p.analyses.Decompiler(f, cfg=cfg.model)
    return d.codegen.text if d.codegen else '// falhou'
if __name__ == '__main__':
    for a in sys.argv[1:]:
        addr = int(a, 16)
        print(f'// ===== {addr:#010x}')
        print(decompile(addr))
