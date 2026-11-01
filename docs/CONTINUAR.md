# Para continuar

Leia antes `CLAUDE.md` e `CHECKPOINT.md`. Endereços abaixo são da 1.2.93.

## Estado
- Branch `migracao-1.2.93`. Os commits `fa48d7a` (Etapa 13 parcial) e `feb22b7` (preparação da missão) são só locais;
  o GitHub está em `05f678a` e não tem `main`.
- Teste diferencial com a preparação feita pelo Rust: 0 divergências em duas partidas (`Mission_Airstrike`/`any30`,
  `Mission_Tides`/`hightide18`). Guia em `ferramentas/diferencial/LEIA-ME.md`.
- Jogo em `C:\Projetos\Into the Breach\Jogo`, cópia de teste em `Jogo_diff`, `libmain.so` em `F:\Downloads\libmain-1.so`.

## Feito: preparação da missão
`Host::load_mission(mapa, setor, tipo_setor, missão)` segue `BoardPlayer::LoadMission` 0x580D50 até antes de
`InitializeMission`: `Board::Load`, `SetupBuildingHealth`, `RandomizeEnvironment`. Provas em
`re/evidencias/v1.2.93/rotinas_preparo.md`; código em `itb-core/src/mission_setup.rs` e `itb-lua/src/map.rs`.

## Método (desde 2026-11-01)
- **Fila por uso:** `itb-uso` joga os 85 tipos de missão e lista os stubs que os scripts tocam
  (`docs/relatorios/uso_missao.txt`: 42 stubs). Reconstruir do topo para baixo, em blocos, e rodar de novo; a lista muda
  porque cada stub que devolvia `nil` escondia o resto do script (ver os erros no fim do relatório).
- **Rodada contra o jogo em um comando:** `rodada.py` em segundo plano e um lote de cliques (abaixo).
- **Ler primeiro o `.so`** (`re/tools/chamados_android.py`), confirmar no PC só o que muda o resultado.
- Documentar ao fechar o bloco, não a cada rotina.

## Próximo passo
0. Bloco 5 da fila de uso (1 a 4 feitos, ver `rotinas_fila_uso.md`; 14 stubs restantes, poucos usos cada):
   `BoardPawn:IsAvoidingMines`, `SetShield`, `SetPowered`, `SetAcid`, `SetActive`, `SetMissionCritical`;
   `Board:IsEnvironmentDanger`, `IsWall`, `SetWall` (o núcleo já tem `set_wall`), `AddShield`, `MarkSpaceImage`,
   `AddTeleport`; `GameMap:TriggerSound`. Depois, o turno inimigo: ler `so_analise/ia_inimiga_pc.md`,
   `ia_inimiga_formulas.md`, `fila_ataque_pc.md` e `turno_pc.md`. A IA começa cada turno com `srand(aiSeed)`
   (AiController+0x80, sorteado no construtor). Para o teste, o Lua do jogo tem `io` (a coleta já escreve arquivo),
   então pode ler `aiSeed` de `saveData.lua` depois que o motor salvar; falta saber quando o motor salva.
   `SelectMap` está lido em `so_analise/inicio_missao_pc.md`.
1. `BoardPlayer::SelectMap` 0x582A60: hoje o mapa é argumento, e o teste o acha pelo terreno. Ela chama Lua
   (`GetMapTag`, `GetMapVetoes`, `GetMap`, `RandomMap`) e gasta `rand()` antes do primeiro ponto de sincronia da
   preparação. Reconstruir e conferir: o ponto `01` do cabeçalho do `itb-diff` deve passar a ficar em sincronia.
   O nome do mapa não sai pela Lua (o envoltório de `AddMap` não pegou, e a global do mapa não muda de tabela): vale
   descobrir o motivo lendo `Board::Load` 0x565860 (0x53A950 chama `AddMap`).
2. `InitializeMission` 0x583130 (`SetBoard`, depois `BaseStart`) e o resto de `BoardPlayer` (`Start`, `StartState`,
   posicionamento dos mechs, turno).
3. Quem grava o setor (string global 0x8D1A88) e o argumento `int` de `LoadMission` (0x65DCF8 passa `[ebp+8]`).

## Como testar contra o jogo
- `python ferramentas/diferencial/rodada.py ../Jogo ../Jogo_diff --modo missao --tempo 600` em segundo plano. Ele
  prepara, roda, compara e grava o relatório.
- Controle do computador: `request_access` com `Breach.exe` depois que a janela abrir. Clicar na janela antes da tecla
  (a primeira tecla falhou por foco). Sequência, em coordenadas da tela 1456x819 do jogo em janela: tecla na tela de
  `Turret_Base`; Novo jogo (356,351); Iniciar (1004,186); Iniciar da dificuldade (1072,598), Normal já marcada e
  conteúdo avançado desligado; ilha verde (520,250). O jogo fecha sozinho depois da coleta. Num só `computer_batch`:
  esperar 12 s, clicar (728,500), tecla espaço, esperar 5 s, (356,351), 3 s, (1004,186), 3 s, (1072,598), 6 s, (520,250).
- Testes com o jogo: `ITB_GAME_DIR` precisa de caminho absoluto (o teste roda dentro de `crates/itb-lua`).

## Depois
Aplicação de efeitos no teste diferencial, laço de quadro, turno inimigo.
