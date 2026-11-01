# Ajudantes de leitura de assembly

Defina `ITB_EXE` com o caminho do `Breach.exe` (ex.: `set ITB_EXE=C:\...\Breach.exe` no Windows).

- `python ann.py 5aba90` — desmonta até o `int3`, anotando strings citadas.
- `python xdisp.py 978 c84` — instruções que usam esses deslocamentos (ex.: campos de `Pawn`) e a função de cada uma.
- `python decomp.py 5aba90` — decompilação com angr (só apoio; confira sempre no assembly).
- `pe.py` — leitura do PE (seções, `dis`, `cstr`, `xrefs`).
