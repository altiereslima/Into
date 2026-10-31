# Etapa 06 — caminhos e alcance (pathfinding)

Com ocupação e `IsBlocked` reconstruídos, o próximo bloco natural é o que o motor usa para mover unidades.

1. **Pathfinding do Board** (assinaturas pelo RTTI em `evidencias/lua_api_v2_all.csv`):
   `GetSimplePath(Point,Point)` → `0x005731F0` (thunk, WeaponTarget slot 0x80),
   `GetPath(Point,Point,int)` → `0x005733D0`, `GetReachable(Point,int,int)` → `0x00573280`,
   `GetSimpleReachable`, `GetDistance`. Usam `GridSearchable` (Board@0); mapear a busca (BFS/A*), ordem de
   vizinhos e desempate — é isso que torna o comportamento idêntico ao original.
2. **Fechar `Pawn::IsCorpse`**: reconstruir `0x00639E10` e nomear `[+0xF80]`, `[+0x9E0]`, `[+0x10E8]` e a
   lista global `0x008CF84C`.
3. **Perfil de caminho do Pawn**: `GetPathProf`, `IsFlying`, `IsJumper`, `IsTeleporter`, `IsBurrower`,
   `GetMoveSpeed`/`GetBaseMove` — alimentam o passo 1.
4. Nomear `pathProf` 5 e 8 (aparecem na tabela do buraco em `0x005AE510`).
5. Opcional: harness diferencial (jogo original sob Windows/Wine + mod Lua que imprime `Board:IsBlocked`,
   `GetPath`... num tabuleiro conhecido) para comparar com o Rust.
6. Em cada entrega: `python tools/map_lua_api_v2.py`, `python tools/check_inventory_v2.py`, `cargo test`,
   e citar N/M/K/% como no CHECKPOINT.

## Reproduzir
```bash
pip install -r tools/requirements.txt          # capstone; llvm-undname opcional (assinaturas C++)
python tools/map_lua_api_v2.py /caminho/Breach.exe evidencias/
python tools/check_inventory_v2.py /caminho/Breach.exe /caminho/Into.the.Breach.v1.2.88.zip
python tools/compare_script_calls_v2.py /caminho/Into.the.Breach.v1.2.88.zip evidencias/lua_api_v2_all.csv evidencias/lua_calls_v2_unmatched.csv
python tools/dump_disasm_etapa05.py /caminho/Breach.exe evidencias/etapa05_disasm
cargo test
```
