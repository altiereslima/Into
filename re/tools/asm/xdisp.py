import capstone,struct,sys
from pe import *
st=text[1]; sz=text[2]; o=text[3]
md2=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
code=data[o:o+sz]
def fs(va):
    oo=off(va); i=oo
    while i>oo-0x8000:
        if data[i-1]==0xCC and data[i]!=0xCC: return va_of(i)
        i-=1
def hits_for(disp):
    pat=struct.pack('<I',disp); res=[]; i=0
    while True:
        i=code.find(pat,i)
        if i<0: break
        for back in range(1,8):
            ins=list(md2.disasm(code[i-back:i-back+16],st+i-back,1))
            if ins and ins[0].size>=back+4 and hex(disp) in ins[0].op_str:
                res.append(ins[0]); break
        i+=1
    return res
for d in sys.argv[1:]:
    for ins in hits_for(int(d,16)):
        print(f"{ins.address:08X} fn {fs(ins.address):08X}: {ins.mnemonic} {ins.op_str}")
