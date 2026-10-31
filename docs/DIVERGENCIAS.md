# Divergências e hipóteses conhecidas

Tudo o que o Rust faz diferente do original, ou que depende de hipótese ainda não provada.

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
| Momento dos empurrões | resolvidos depois de uma animação de 0,4 s (`Pawn+0x978`), em paralelo com os próximos efeitos | logo após cada `SpaceDamage` | Ordem de colisões em armas com vários empurrões | Ler o executor do SkillEffect e o laço de atualização |
| Movimentos (`AddMove`, `AddLeap`, `AddCharge`, `AddTeleport`) | executor do SkillEffect, ainda não lido | primeira unidade da casa inicial vai para a última casa do caminho | Casos com casa final ocupada | Ler o executor |
| Efeito de chegada (0x0059C5C0) | água, buraco, lava, ácido, itens, trens, "FireBoost", casos de cadáver | água afoga quem não voa e não tem +0x99D; buraco derruba quem não voa; item vira evento | Unidades especiais | Terminar a leitura |
| `iTerrain = 9` | animação; o buraco aparece depois | buraco imediato | Unidades na casa não caem nesta versão | Ler quem conclui a animação |
| Prédios | rede elétrica, chance de resistir (0x005C4B90), prédios de energia, objetivos | só a vida da casa; dano à rede vira evento | Rede elétrica | Reconstruir a parte do `Game` |
| Unidade morrendo | sólida durante parte da animação de morte (0x0062CB10) | não sólida | Empurrões no mesmo instante | — |
| Instâncias `Board`/`Game` | criadas pelo motor em missão | ainda não existem | Scripts de missão | M3/M4 |
| `user/missionData.lua` | arquivo do usuário, opcional | ausente → ignorado | Nenhum | — |
