import re,sys
from pe import *
def ann(a,n=2000,stop='int3'):
    out=[]
    for l in dis(a,n):
        m=re.findall(r'0x([0-9a-f]{6,8})\b',l)
        extra=''
        for h in m:
            v=int(h,16)
            if 0x800000<=v<0x900000:
                s=cstr(v)
                if s and len(s)>2 and all(32<=c<127 for c in s): extra+='   ; '+repr(s.decode())
        out.append(l+extra)
        if stop and stop in l: break
    return out
if __name__=='__main__':
    for a in sys.argv[1:]:
        print('=====',a)
        print('\n'.join(ann(int(a,16))))
