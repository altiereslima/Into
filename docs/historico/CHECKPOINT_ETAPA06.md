# CHECKPOINT — Etapa 06: mudança de rumo para "motor Rust + scripts originais"

**Data:** 2026-10-30. Histórico das etapas anteriores em `docs/historico/`.

## O que mudou
O projeto deixou de ser "traduzir o `Breach.exe` para Rust" e passou a ser **um motor Rust que executa
os scripts Lua originais** da instalação do usuário (motivos em `docs/ARQUITETURA.md`). O código e as
provas das Etapas 01–05 foram mantidos e reorganizados num workspace:

- `crates/itb-core` (antes `itb-recon`): regras puras + testes.
- `crates/itb-lua` (novo): host Lua 5.1 (mlua, Lua embutida), manifesto da API gerado do EXE,
  binário `itb-boot`.
- `re/` (antes `tools/` + `evidencias/`): ferramentas e provas.

## Números desta etapa
| Métrica | Valor |
|---|---|
| Scripts originais que carregam no host Rust | **137/137** presentes (1 opcional ausente: `user/missionData.lua`), 0 erros |
| Nomes da API expostos à Lua | 475: **34 reconstruídos**, **11 plataforma**, 430 stubs |
| Entrypoints nativos distintos (métodos + globais) | 368 |
| Entrypoints com Rust testado | **47 (12,77%)**: 35 com lógica, 12 acessores triviais; 1 parcial |
| Restantes | 321 (87,23%) |
| Constantes globais com valor extraído | 190 (164 int, 21 cores, 4 vetores, 2 bool) — antes 92 |
| Testes | 53 passam (`cargo test --workspace`) + 1 de integração com o jogo (`--ignored`) |

Observação: os 47 entrypoints contam o que está no `itb-core`; nem todos já estão ligados na Lua
(as rotinas de `Board`/`Pawn` só serão ligadas quando o motor criar essas instâncias — marco M3/M4).

## Novidades de engenharia reversa (provas em `re/evidencias/rotinas_valores.md`)
- `lua_tointeger` do `lua5.1.dll` arredonda ao par (`fistp`): replicado em `lua_number_to_int`.
- `rand()` da CRT MSVC e `random_int`/`random_bool`: sequência idêntica ao jogo para a mesma semente.
- `Point` completo (incluindo `Point()` = 0x80000001, `__mul` com `roundf`, `GetString`), `GetDirection`.
- `Rect2D`, `GL_Color`, `Button` (`ButtonDesc`) com os 5 construtores e campos internos.
- `ScreenSizeX/Y`, `GetDefaultScale`, `GetBoardScale` (fórmulas do EXE; entradas ainda hipotéticas).
- Extrator de constantes generalizado: 5 formas de atribuição em `_G` (antes só 1).

## Próximo passo
M2 (tipos de valor restantes, guiado pelo relatório de stubs: `StyleInfo`, `Text`, `Objective`,
`RegionInfo`, `JoyRumble`, `PilotSkill`) e M3 (descobrir no EXE o ciclo de vida: que funções Lua o
motor chama depois do boot). Ver `docs/ROADMAP.md`.

## Reproduzir
```bash
cargo test --workspace
cargo run -p itb-lua --bin itb-boot -- "/caminho/Into the Breach"
pip install -r re/tools/requirements.txt
python re/tools/map_lua_api_v2.py /caminho/Breach.exe re/evidencias/
python re/tools/check_inventory_v2.py /caminho/Breach.exe /caminho/Into.the.Breach.zip
python re/tools/gen_api_manifest.py      # o CI exige que o manifesto Rust esteja em dia
```
