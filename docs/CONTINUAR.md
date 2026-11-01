# Para continuar

Leia antes `CLAUDE.md` e `CHECKPOINT.md`. Endereços abaixo são da 1.2.93.

## Estado
- Branch `migracao-1.2.93`. O commit `fa48d7a` (Etapa 13 parcial) é só local; o GitHub está em `05f678a` e não tem `main`.
  A preparação da missão (abaixo) está sem commit.
- Teste diferencial com a preparação feita pelo Rust: 0 divergências em duas partidas (`Mission_Airstrike`/`any30`,
  `Mission_Tides`/`hightide18`). Guia em `ferramentas/diferencial/LEIA-ME.md`.
- Jogo em `C:\Projetos\Into the Breach\Jogo`, cópia de teste em `Jogo_diff`, `libmain.so` em `F:\Downloads\libmain-1.so`.

## Feito: preparação da missão
`Host::load_mission(mapa, setor, tipo_setor, missão)` segue `BoardPlayer::LoadMission` 0x580D50 até antes de
`InitializeMission`: `Board::Load`, `SetupBuildingHealth`, `RandomizeEnvironment`. Provas em
`re/evidencias/v1.2.93/rotinas_preparo.md`; código em `itb-core/src/mission_setup.rs` e `itb-lua/src/map.rs`.

## Próximo passo
1. `BoardPlayer::SelectMap` 0x582A60: hoje o mapa é argumento, e o teste o acha pelo terreno. Ela chama Lua
   (`GetMapTag`, `GetMapVetoes`, `GetMap`, `RandomMap`) e gasta `rand()` antes do primeiro ponto de sincronia da
   preparação. Reconstruir e conferir: o ponto `01` do cabeçalho do `itb-diff` deve passar a ficar em sincronia.
   O nome do mapa não sai pela Lua (o envoltório de `AddMap` não pegou, e a global do mapa não muda de tabela): vale
   descobrir o motivo lendo `Board::Load` 0x565860 (0x53A950 chama `AddMap`).
2. `InitializeMission` 0x583130 (`SetBoard`, depois `BaseStart`) e o resto de `BoardPlayer` (`Start`, `StartState`,
   posicionamento dos mechs, turno).
3. Quem grava o setor (string global 0x8D1A88) e o argumento `int` de `LoadMission` (0x65DCF8 passa `[ebp+8]`).

## Como testar contra o jogo
- `python ferramentas/diferencial/preparar.py ../Jogo ../Jogo_diff --modo missao`, depois
  `python ferramentas/diferencial/rodar.py ../Jogo_diff --tempo 900` em segundo plano.
- Controle do computador: `request_access` com `Breach.exe` depois que a janela abrir. Clicar na janela antes da tecla
  (a primeira tecla falhou por foco). Sequência, em coordenadas da tela 1456x819 do jogo em janela: tecla na tela de
  `Turret_Base`; Novo jogo (356,351); Iniciar (1004,186); Iniciar da dificuldade (1072,598), Normal já marcada e
  conteúdo avançado desligado; ilha verde (520,250). O jogo fecha sozinho depois da coleta.
- `cargo run -p itb-lua --bin itb-diff -- ../Jogo ../Jogo_diff/diff_original.txt`.
- Testes com o jogo: `ITB_GAME_DIR` precisa de caminho absoluto (o teste roda dentro de `crates/itb-lua`).

## Depois
Aplicação de efeitos no teste diferencial, laço de quadro, turno inimigo.
