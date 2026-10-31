# Etapa 06 — tipos de valor, aleatoriedade, tela e constantes

Breach.exe v1.2.88 (SHA256 `4bd9e4cc…84b0`). Desmontagens reproduzíveis com `re/tools/dump_ctor_bodies.py`
e capstone; endereços são VAs.

## Point
| Lua | VA | Comportamento |
|---|---|---|
| `Point()` | construct → 0x00465B00 | `x = y = 0x80000001` |
| `Point(x,y)`, `Point(Point)` | — | cópia dos inteiros |
| `__add` / `__sub` | 0x004520A0 / 0x00452020 | soma/subtração de 32 bits, empilha novo Point |
| `__eq` | 0x00451FE0 | x e y iguais |
| `__mul(int)` | 0x00451F60 | `(int)roundf((float)v * (float)k)` por componente (`roundf` da UCRT em 0x00780857) |
| `Manhattan` | 0x004BC350 | `abs(x-px) + abs(y-py)` |
| `GetString` / `GetLuaString` | 0x004BC380 / 0x004BC510 | `"Point( " + x + ", " + y + " )"` (strings 0x00822174, 0x0081CEBC, 0x00822170) |

## Rect2D (construct 0x00465D00 / 0x00465DB0 / 0x00465E64)
`Rect2D()` = (-1,-1,0,0). `(int,int,int,int)`, `(Point,int,int)` e `(Rect2D)` compartilham o mesmo corpo
(dobra de código idêntico) e copiam 4 inteiros; `(Point,Point)` copia `p.x, p.y, s.x, s.y`.

## GL_Color
`GL_Color()` = (1,1,1,1) (0x00466490); `(r,g,b)` → alfa 1.0 (0x00466610); `(r,g,b,a)` (0x00466540).
`Normalize` 0x0046C510 divide r,g,b por 255.0 (constante 0x00838028); `WithAlpha` 0x0046C4E0.

## Button (C++ `ButtonDesc`, 0x54 bytes)
Campos: image +0x00 (string), pos +0x18, label +0x20 (string), hitstats +0x38 (Rect2D), +0x48, +0x4C (9), +0x50 (bool).
Ctors C++: `()` 0x004692D0, `(image,Point)` 0x004691D0, `(image,Point,bool)` 0x004690D0,
`(Rect2D,label)` 0x00468FD0 (+0x48 = 1), `(Rect2D,label,int)` 0x00468EE0 (+0x4C = int; +0x48 não inicializado).

## GetDirection 0x004B2340
(0,0) → 4. Senão eixo dominante (empate → Y), normaliza ±1, procura na tabela 0x008C6800
preenchida pelo inicializador estático 0x00402FB0: UP(0,-1), RIGHT(1,0), DOWN(0,1), LEFT(-1,0).

## Aleatoriedade
`rand` 0x0078478E: `seed = seed*0x343FD + 0x269EC3; return (seed>>16) & 0x7FFF` (por thread); `srand` 0x007847AF.
`random_int(n)` 0x004DFD50: n==0 → 0, senão `rand() % n`. `random_int(a,b)` 0x004DFD70: a==b → a, senão `a + rand() % (b-a)`.
`random_bool(n)` 0x004DFDE0: `rand() % n == 0`. `random_bool(a,b)` 0x004DFE00: `rand() % b < a`.

## Tela
`ScreenSizeY` 0x004B25F0, `ScreenSizeX` 0x004B2560, `GetDefaultScale` 0x004D47F0 (limiares 1070, 1420, 1440, 2560 → 2/3/4),
`GetBoardScale` 0x004D4640 (`min(config "max_scale" [padrão 5?], GetDefaultScale())`). Globais lidas: 0x0088F10C, 0x0088F114, 0x0088EB4C.

## lua5.1.dll
`lua_tointeger` (RVA 0x16D0): `fld qword; fistp dword` → arredondamento da FPU (par mais próximo por padrão).

## Constantes globais (`lua_global_values.csv`)
Cinco formas de atribuição em `_G` reconhecidas: `lua_pushinteger` imediato, `operator=(int const&)` 0x004508B0,
`(Point)` 0x00450910, `(GL_Color)` 0x004509E0, inteiros/booleanos via variável local (0x006802F0, 0x00680B90, 0x00680D00),
e ponteiro de instância (0x00680AE0, `PAWN_FACTORY`). Resultado: 164 inteiros, 21 cores, 4 vetores, 2 booleanos,
1 instância; 5 usos são só leitura (`ANIMS`, `Profile`, `zones`, `tags`, `PawnList`).
