# Divergências e hipóteses conhecidas

Tudo o que o Rust faz diferente do original, ou que depende de hipótese ainda não provada.
Os endereços desta tabela são da 1.2.88; a tradução para a 1.2.93 está em `re/evidencias/v1.2.93/migracao_enderecos.csv`.

| Área | Original | Rust hoje | Impacto | Como fechar |
|---|---|---|---|---|
| Conversão número→int **dentro das bibliotecas Lua** (`string.sub`, `table.insert`...) | `lua5.1.dll` arredonda ao par (`fistp`) | Lua embutida (mlua, x86-64) trunca | Só com argumentos fracionários; raro | Compilar a Lua com `lua_number2int` = arredondamento, ou trocar por uma Lua com essa opção |
| Conversão na **API nativa** | arredonda ao par | igual (`lua_number_to_int`) | — | — |
| Membros de classe devolvidos por propriedade (`btn.pos`, `btn.hitstats`) | luabind devolve referência: `btn.pos.x = 1` altera o botão | devolve cópia | Nenhum script base faz isso (verificado por busca) | Userdata com referência compartilhada |
| `ScreenSizeX/Y`, `GetDefaultScale` | lê globais 0x0088F10C, 0x0088F114, 0x0088EB4C | supõe altura da janela e razão de aspecto | Valores de UI | Rastrear quem escreve essas globais |
| `GetBoardScale` | `min(config "max_scale", padrão)`; padrão suposto 5 | 5 | Escala do tabuleiro | Confirmar 0x004D3030 |
| `ButtonDesc +0x48` | não inicializado em 4 de 5 construtores | `None` | Nenhum conhecido | — |
| `Pawn::IsCorpse` | chama 0x00639E10(12) | resultado é dado de entrada | Ocupação por cadáveres | Reconstruir 0x00639E10 |
| Cache do Dijkstra (`Board+4`) | `GetReachable` reaproveita o resultado por (origem, alcance, perfil); `DamageSpace` limpa o cache (0x004CF5D0) | recalcula sempre | Só diverge se o tabuleiro mudar por outro caminho sem limpar o cache | Achar os outros pontos que limpam Board+4 |
| Consultas de Pawn ainda não reconstruídas | `IsFlying`, blindagem 0x0063C100, escudo 0x0062F2F0, `IsBoosted` (habilidade 11, "Arrogant_Boost"), `GetBaseMove`, `IsAbility`, propriedades do tipo ("Jumper", "Teleporter", "Burrows") | dados de entrada em `PawnState` | `IsDeadly`, `GetPathProf`, `GetMoveSpeed` dependem delas | Reconstruir a leitura das tabelas de tipo (0x004493D0, 0x0063AB20/0x0061D180) |
| `IsPassiveSkill` | lê string global 0x008D0E94 e lista 0x008D0E48 | configuração do host | Armas com passivas | Reconstruir 0x00625320 e quem preenche a lista |
| Relógio do executor | cronômetros descontam `g_8B6AE0 × 0.0625` por quadro (soma móvel do tempo de quadro) | segundos contínuos; empurrão 0,4 s e corpo a corpo 0,15 s (valores do EXE); quem vence no mesmo instante: unidades primeiro, por ordem de criação | Empates de tempo | Ler 0x0048F5C9 e a ordem unidade/tabuleiro no quadro |
| Tempo de voo | atualização de cada projétil (0x0055D0A0, 0x0055CFA0, 0x0055D260...) | projétil 0,25 s, artilharia/aéreo 0,5 s (SUPOSIÇÃO, `Timing`) | Ordem relativa a empurrões e atrasos curtos | Ler as rotinas de atualização dos projéteis |
| `Board:AddEffect` na Lua | enfileira e roda ao longo dos quadros | roda a fila até ficar ociosa, na hora | Scripts que leem o tabuleiro logo depois | Laço de jogo (M7) |
| Movimentos (`AddMove`, `AddLeap`, `AddCharge`, `AddTeleport`) | animados (0x00632670, 0x00632750, 0x006329F0, +0x10B4, 0x00636610) | `GetPawn(caminho[0], false)` chega na última casa na hora | Ocupação durante o trajeto | Ler as animações de movimento |
| Efeito de chegada (0x0059C5C0) | água, buraco, lava, ácido, itens, trens, "FireBoost", casos de cadáver | água afoga quem não voa e não tem +0x99D; buraco derruba quem não voa; item vira evento | Unidades especiais | Terminar a leitura |
| `iTerrain = 9` | animação; o buraco aparece depois | buraco imediato | Unidades na casa não caem nesta versão | Ler quem conclui a animação |
| Prédios | rede elétrica, chance de resistir (0x005C4B90), prédios de energia, objetivos | só a vida da casa; dano à rede vira evento | Rede elétrica | Reconstruir a parte do `Game` |
| Unidade morrendo | sólida durante parte da animação de morte (0x0062CB10) | não sólida | Empurrões no mesmo instante | — |
| Instâncias `Board`/`Game` | criadas pelo motor em missão | ainda não existem | Scripts de missão | M3/M4 |
| `user/missionData.lua` | arquivo do usuário, opcional | ausente → ignorado | Nenhum | — |
| Regras lidas na 1.2.88 | versão atual é a 1.2.93 | código não relido rotina a rotina. O teste diferencial cobre valores, unidades, consultas de casa, alcance, caminho e efeitos gerados pelas armas, mas não a aplicação de dano, empurrão e o executor | Mudança do mesmo tamanho em rotina fora do teste passaria despercebida | Estender o teste diferencial à aplicação de efeitos |
| `IgnoreFlip`, `Pushable` | `Pawn::OnInit` lê da definição do tipo | dados de entrada em `PawnStatusBytes` | Unidades reais ainda não vêm de `pawns.lua` | M4 |
| Ordem de `MAP_LIST` | ordem da listagem da pasta feita pelo motor | alfabética (SUPOSIÇÃO: é o que o NTFS costuma devolver) | Escolha de mapa por sorteio, se a ordem diferir | Ler a rotina de listagem; comparar `MAP_LIST` no teste diferencial |
| `AddPawn`/`SpawnPawn` com nome de zona | escolhem a casa pela zona | stub que devolve nil | Scripts que usam essas formas (ex.: `"disposal"`) | Ler 0x56EE40, 0x56ECE0, 0x5690E0, 0x569250 |
| `SpawnPawn` em água, unidade que não voa | caminho com `general_DiamondTarget` | entra na casa com estado 1 | Inimigo em água | Ler 0x568D5A em diante |
| `GetDropZone` com menos de 3 casas | caminho em 0x560E3A | devolve o que houver | Mechs em mapas apertados | Ler o resto da rotina |
| Teste de casa para os times 3 e -3 | caminho próprio com `GetDropZone` | recusa | Quem chamar com esses times | Ler 0x57309B em diante |
| `Board+0x2CA8` | muda o sorteio e `SpawnQueued` | sempre falso | Não identificado | Achar quem grava |
| Energia ao criar `Game` | 5/7 medido no jogo, dificuldade Normal | 5 fixo | Outras dificuldades podem diferir | Medir nas outras dificuldades; ler a rotina que grava |
| `Board::RandomizeTerrain` e o resto da preparação do mapa | antes de `BaseStart`, chão vira floresta (6 casas em `any38`, 4 em `anyAE3`) | não existe | Terreno difere do arquivo `.map`; o teste diferencial parte do terreno real | Ler `BoardPlayer::RandomizeEnvironment` e `Board::RandomizeTerrain` |
| `IsNewEnemies`, `IsNewEquipment`, `IsNewMissions` | opções "Conteúdo avançado" da partida (lidas de 0x4D3E10) | `HostConfig`, ligadas por padrão | Listas de inimigos, armas e missões | Ler a configuração real |
| Dois dos 3 `rand()` de `CreatePawn` | em construtores não localizados | só o consumo; os valores são descartados | Campos que usem esses valores, além de +0x924 | Localizar as chamadas |
| Perfil do jogador em `IsIslandUnlocked` | `perfil+0x30` | 0 | Ilhas e inimigos liberados | Ler `GameProfile` |
| Turno em `Game` | `BoardPlayer` | não existe ainda: time 2, turno 0 | Scripts que olham o turno | Reconstruir `BoardPlayer` |
| `StartTurn` em `SpawnPawn` do jogador | `Pawn::StartTurn(0)` | não chamada | Estado de início de turno | Ler `Pawn::StartTurn` |
| Lado visual e sonoro do surgimento | `SpawnAnimation`, sons, conquistas, voz | não modelado | Só visual e som, salvo o que `SpawnAnimation` gravar | Ler 0x637070 |
| `AddPawn` de unidade que já está em tabuleiro | remove do tabuleiro anterior (+0x944) | só aceita unidade recém-criada; outro tabuleiro não é limpo | Mover unidade entre tabuleiros | Ler 0x625DC0 (1.2.93) |
| Propriedades de unidade sem uso no Rust | `Leader`, `Minor`, `Class`, `Image`, `Tooltip`, `GetExtraSpaces`... | ignoradas | Unidades grandes, líderes, visual | Ligar ao reconstruir quem as lê |
| `IsArmor`, `IsFireImmune` | combinam propriedade, piloto e passivas | só `Armor` e `IgnoreFire` | Pilotos e passivas | Reconstruir as duas rotinas |
| `Game` | objeto real do motor | 5 acessores reais; o resto é stub (`AddObjective`, dicas, voz...) | Objetivos e interface | Reconstruir `GameMap` |
| `IsAbility`, `Road_Runner` e demais dados de piloto | vêm do piloto (`SetPilot`) | unidade nasce sem piloto | Mechs com habilidades | Reconstruir `Pilot` |
| Vida dos prédios no início da missão | `Board::SetBuildingHP` ajusta depois da carga (SUPOSIÇÃO pelo nome) | fica o valor do mapa (4) | Rede elétrica e dano a prédios | Ler `Board::SetBuildingHP` e o início de missão |
| Chaves de jogo salvo em `LoadScript` | `cracked`, `vines`, `item`, `rain`, `teleports`... | relatadas como não aplicadas | Só ao carregar jogo salvo | Salvar e carregar (M9) |
| Efeitos visuais da carga | `UpdateBuildingImage`, `StartSavePeopleAnim` | não modelados | Só visual | Render (M8) |
| `Pawn::SetGuard(false)` em `Pawn::Push` | grava +0x8D2 no caminho estável | não modelado | Nenhum conhecido: na 1.2.93 só há escritas nesse deslocamento | Reavaliar se surgir uma leitura indireta |
