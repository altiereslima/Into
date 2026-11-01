# Instruções para o Claude (Claude Code / Cowork)

Leia este arquivo inteiro antes de começar. Depois leia `CHECKPOINT.md` (estado atual), `docs/ROADMAP.md`
(marcos), `docs/DIVERGENCIAS.md` (o que ainda difere do original) e `docs/decisoes/`.

## Idioma e estilo
- Fale e escreva **em português** com o usuário (Altieres). Ele gosta de Python para ferramentas.
- Respostas curtas; relatórios de etapa com tabela de números (antes/depois) e "próximo passo".

## O projeto
Reimplementação do motor do **Into the Breach** em Rust que executa os **scripts Lua originais** da
instalação do usuário (modelo OpenMW/OpenRCT2). As regras são reconstruídas lendo o assembly do
`Breach.exe` (PC, **v1.2.93**, igual à versão Netflix). As provas antigas foram lidas na 1.2.88.

**Endereços:** os `0x…` em `re/evidencias/rotinas_*.md` e nos comentários do Rust são da 1.2.88. Para achar um deles
na 1.2.93, consulte `re/evidencias/v1.2.93/migracao_enderecos.csv`. O que for lido daqui em diante é da 1.2.93: diga a
versão junto do endereço e prefira o nome real. Em `CHECKPOINT.md`, `docs/*.md`, `crates/` e `re/evidencias/*.md`,
`migrar_enderecos.py` toma todo `0x00XXXXXX` como endereço da 1.2.88; escreva os novos sem os zeros (`0x62DAB0`).

- `crates/itb-core`: regras em Rust puro (tabuleiro, ocupação, pathing/pathfinding, dano, aplicação de
  efeitos, executor com relógio).
- `crates/itb-lua`: host Lua 5.1 (mlua) que expõe a API nativa aos scripts; binários `itb-boot` e `itb-weapon`.
- `re/tools`: ferramentas Python de engenharia reversa (só leitura dos binários).
- `re/evidencias`: provas de cada rotina (`rotinas_*.md`), inventário e nomes da 1.2.88.
- `re/evidencias/v1.2.93`: inventário atual (`lua_api_v2_all.csv`), nomes, tabela de migração e `LEIA-ME.md`.

## Regras invioláveis
1. **Nenhum arquivo do jogo no repositório**: EXE, DLLs, `scripts/`, `maps/`, `resources/`, `.bank`,
   `libmain.so`, PNGs/sons extraídos. Só código, nomes, endereços e documentação.
2. **Nunca gravar tokens/senhas** em arquivos, commits ou config do git. (Um token já foi exposto no chat
   e o usuário foi orientado a revogá-lo.)
3. Cada afirmação sobre o original precisa de prova: endereço + leitura do assembly (capstone; angr só
   como apoio, sempre conferido no asm). Suposições ficam marcadas como SUPOSIÇÃO e vão para
   `docs/DIVERGENCIAS.md`.
4. Fidelidade antes de beleza: reproduzir inclusive desempates, ordens de iteração, arredondamentos
   (`lua_tointeger` arredonda ao par; `rand()` do MSVC; heap do `std::priority_queue` do MSVC...).
5. Commits terminam com:
   ```
   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

## Como trabalhar
- Jogo do usuário: pasta da instalação (ex.: `C:\Program Files (x86)\Steam\steamapps\common\Into the Breach`).
- Testes: `cargo test --workspace` (sem o jogo) e `ITB_GAME_DIR=<jogo> cargo test -p itb-lua -- --ignored`.
- Boot e cobertura: `cargo run -p itb-lua --bin itb-boot -- <jogo> --cobertura re/evidencias/cobertura_api.tsv`.
- Armas: `cargo run -p itb-lua --bin itb-weapon -- <jogo> --todas` (varredura) ou `... <jogo> Prime_Punchmech 5 3 5 4`
  (uma arma, com linha do tempo e diferença do tabuleiro).
- Mapas: `cargo run -p itb-lua --bin itb-map -- <jogo> acid0` ou `... <jogo> --todos`.
- Teste diferencial: `ferramentas/diferencial/LEIA-ME.md`. Sempre rode o jogo por `rodar.py`, que protege a pasta de dados do usuário.
- Inventário: `python re/tools/map_lua_api_v2.py <Breach.exe> re/evidencias/v1.2.93/` e
  `python re/tools/gen_api_manifest.py` (o CI confere que `api_manifest.rs` está atualizado).
- Conferência na 1.2.93: `python re/tools/conferir_1293.py <Breach.exe>`.
- Ao reconstruir uma rotina: atualizar `re/tools/rust_status.json` (done/partial), regenerar inventário,
  `clippy` sem avisos, testes novos, `rotinas_*.md`, `CHECKPOINT.md` (mover o anterior para `docs/historico/`),
  relatórios em `docs/relatorios/`.
- Assembly: `re/tools/asm/` (ann.py, xdisp.py, decomp.py; variável `ITB_EXE`).
- Nomes: `python re/tools/match_android_symbols.py <Breach.exe> <libmain.so> re/evidencias/v1.2.93/` e
  `python re/tools/gen_nomes_resumo.py`. Prefira o **nome real** (`Pawn::Damage`) ao endereço nos textos novos.

## Fila de trabalho (ver CHECKPOINT.md)
Ordem escolhida pelo usuário em 2026-11-01: regras antes de gráficos, para chegar ao teste diferencial.
1. Etapa 13, lida direto na 1.2.93: terminar M4 (o resto de `BoardPlayer`: `SelectMap`, `InitializeMission`, `Start`,
   `StartState`, posicionamento; a preparação com terreno e vida dos prédios já está em `Host::load_mission`, e
   `startNewGame` e `BaseStart` já rodam; `itb-map <jogo> <mapa> --missao <Tabela>` mostra o estado). A cada regra nova, estender `ferramentas/diferencial/coleta.lua` e comparar com o jogo
   real: é a prova mais forte que o projeto tem. Depois, laço de quadro
   (`CApp`/`Game`/`Board`/`Pawn::OnLoop`), `BoardPlayer`, tempo de voo dos projéteis, efeito de chegada completo
   (`BoardSpace::CheckTerrainDangers`) e turno inimigo (M6).
2. Gráficos e som (decisão 0001): `itb-assets` lê `resource.dat` e `.bank`, extrai localmente com índice;
   som sem depender do FMOD (só 20 funções da API são usadas: ver `re/evidencias/fontes_externas/vita_loader.md`).
3. Trocar os nomes provisórios do código pelos reais (`Tile::Push`, `byte_0x91e`...).
4. Os idiomas são tratados por outra IA, fora deste trabalho.
