import pefile, struct, capstone, sys
import os; EXE=os.environ.get('ITB_EXE','Breach.exe')
pe=pefile.PE(EXE, fast_load=True)
data=open(EXE,'rb').read()
base=pe.OPTIONAL_HEADER.ImageBase
secs=[(s.Name.rstrip(b'\0').decode(),base+s.VirtualAddress,s.SizeOfRawData,s.PointerToRawData,s.Misc_VirtualSize) for s in pe.sections]
text=[s for s in secs if s[0]=='.text'][0]
def off(va):
    for n,st,sz,o,vs in secs:
        if st<=va<st+sz: return o+va-st
    return None
def va_of(o):
    for n,st,sz,fo,vs in secs:
        if fo<=o<fo+sz: return st+o-fo
def cstr(va,n=80):
    o=off(va)
    if o is None: return None
    return data[o:o+n].split(b'\0')[0]
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32); md.detail=False
def dis(va,n=60,stop_ret=False):
    o=off(va); out=[]
    for i in md.disasm(data[o:o+n*16],va):
        out.append(f"{i.address:08X}: {i.mnemonic} {i.op_str}")
        if len(out)>=n: break
        if stop_ret and i.mnemonic in('ret','int3'): break
    return out
def find_str(s):
    res=[];i=0
    pat=s.encode()+b'\0'
    while True:
        i=data.find(pat,i)
        if i<0: break
        if i==0 or data[i-1]==0: res.append(va_of(i))
        i+=1
    return res
def xrefs(va):
    pat=struct.pack('<I',va); res=[]; i=text[3]
    end=text[3]+text[2]
    while True:
        i=data.find(pat,i,end)
        if i<0:break
        res.append(va_of(i)); i+=1
    return res
