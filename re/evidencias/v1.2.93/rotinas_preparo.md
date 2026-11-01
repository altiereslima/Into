# Preparação da missão: leitura na 1.2.93

Todos os endereços são da 1.2.93. Código: `itb-core/src/mission_setup.rs` e `Host::load_mission` em `itb-lua/src/map.rs`.
Teste contra o jogo: `ferramentas/diferencial/` (seções `preparo`, `casa` e `com_unidades`).

## `BoardPlayer::LoadMission` (0x580D50)
Argumentos, na ordem da pilha: `int` (gravado em BoardPlayer+0xFEC), `int setor`, `string missão` (BoardPlayer+0xFD4) e
`string`, que vai para `SelectMap`. Chamadores: `GameMap::CheckBattle` (0x60D3A3, `Mission_Test`; 0x60D4DE, tutorial) e
0x65DCF8 (partida normal).

1. `SelectMap` 0x582A60 (não lida).
2. `Board::Load` 0x565860: executa `maps/<nome>.map`, chama `AddMap`, `LoadScript` 0x562220, 0x573A20 e 0x573260.
3. `SetupBuildingHealth(setor)` 0x591540.
4. Se a missão não é `"Mission_ActiveTutorial"` (0x47C7C0): `RandomizeEnvironment` 0x58B380.
5. `InitializeMission` 0x583130.

O nome da missão é o que `CallMethod` resolve: na partida, "Mission6", que `GAME:GetMission` acha em `GAME.Missions[6]`.

## `Board::Load`, depois de `LoadScript`
- 0x573A20: para cada casa, guarda em +0x8E4 + 4·dir o terreno da vizinha em cada direção (tabela 0x8D1A40). Só gráficos.
- 0x573260: zera a lista Board+0x2C44 e acrescenta `Point(x, y)` das casas com o campo +0x2AE0 == 1, em x e depois y.
  `Board:SetTerrain` (0x565E90) também a refaz, depois de `BoardSpace::SetTerrain` 0x5A1B10.

## `SetupBuildingHealth(int setor)` (0x591540)
`getBuildingHealth(setor)` pelo ajudante 0x597890 (função Lua global → `std::vector<int>`), depois `Board::SetBuildingHP`
0x56D260 com o vetor.

## `Board::SetBuildingHP(std::vector<int> v)` (0x56D260)
1. Para cada `v[i]` (0x56D2C0..0x56D30A): `n` = tamanho da lista de prédios (`(fim − início) >> 3`), convertido como
   inteiro sem sinal (0x839A70 = {0, 2^32}); `x = (float) v[i] / 100.0f * (float) n` (`divss` por 0x839010 = 100.0f,
   `mulss`); 0x476150 passa `x` a `double`, chama 0x799B30 e volta a `float`; `cvttss2si` trunca.
   0x799B30 é o `ceil` da CRT em SSE2: constantes 0x7F8A30 = 1.0, 0x7F8A40 = 0x433, 0x7F8A50 = 0.0, 0x7F8A60 = 0x7FF,
   0x7F8A70 = −0.0; para positivos soma 1.0 quando `x > trunc(x)`; entre −1 e 0 devolve −0.0.
   Por ser em `float`, `ceil(0.6f · 25)` = 16, onde a conta exata daria 15.
2. Para cada prédio da lista (0x56D3A3..0x56D419): `esi` avança até a primeira conta não nula, sem voltar, e para em 4
   (constante, não o tamanho do vetor). Sem conta restante, sai. Senão: casa por 0x561CE0; `SetTerrain(1)` (0x5A1B10);
   `Health(i + 1)` (0x4E0CE0) gravada por 0x5AF190; com vida atual (+0x78) 0, `SetTerrain(2)`; `conta[i] -= 1`.
3. Para x, depois y (0x56D441..0x56D52F), casa por 0x55EAB0; com +0x2AE0 == 2 (entulho): `rand() % 100`
   (`rand` 0x7855E3, `idiv` por 100); `m` = 1 + o primeiro índice em que `r < soma acumulada`, ou 1 se nenhum (ou vetor
   vazio); `Health(0, m)` (0x4E0D80: atual 0, máximo `m`) gravada por 0x5AF190, que deixa o entulho com atual 0.

## `BoardPlayer::RandomizeEnvironment` (0x58B380)
`setor` é a `std::string` global 0x8D1A88 ("grass", "sand", "snow", "acid"...).
1. Tabela 0x8392D0 = {6, 7, 5, 12, 8, 9}; o laço usa as 4 primeiras. Para cada `tipo`: `getEnvironmentChance(setor, tipo)`
   pelo ajudante 0x597060 e `Board::RandomizeTerrain(tipo, chance)`.
2. 0x449640: `CallMethod(BoardPlayer+0xFD4, "IsCracksAllowed")` por `lua_getfield(LUA_GLOBALSINDEX, "CallMethod")` e
   `lua_pcall(2, 1)` (0x449B40, 0x4497A0). Só um booleano vale (`lua_type == 1`, depois `lua_toboolean`); outro tipo
   ou erro cai no `catch`, mostra "Something went wrong in <obj>::IsCracksAllowed" e devolve falso.
   Se verdadeiro: `getCrackChance(setor)` (0x597190) e `RandomizeTerrain(16, chance)`.
3. `getRainChance(setor)` (0x597190); `rand() % 100 < chance` sempre gasta o `rand()`. Se verdadeiro:
   `Board::SetWeather(3, tipo, Point(0, 0), Point(8, 8), -1.0f)` (0xBF800000), com `tipo` 2 se o setor tem 4 letras e é
   "snow", 1 se é "acid", 0 nos outros (comparação 0x46C1D0 e tamanho 4).

Os ajudantes 0x597060 e 0x597190 mostram "Something went wrong in <typename A,B,C> function" e devolvem 0 em erro.

## `Board::RandomizeTerrain(int tipo, int chance)` (0x568940)
Exposta à Lua. Chance 0: nada. Terreno-base: 3 (água) se `tipo == 5`, senão 0. Para x, depois y, casa por `IsValid`
(slot 8) ou sentinela: se +0x2AE0 é o terreno-base, `rand() % 100`; se o resultado é menor que a chance e o tamanho da
`std::string` +0x27B4 (`custom`, tamanho em +0x27C4) é 0, `BoardSpace::SetTerrain(tipo)` (0x5A1B10). A lista de prédios
não é refeita.

## `Board::SetWeather` (0x56D580) e `Board:GetHealth` (0x55FFB0)
`SetWeather` repassa os 5 argumentos a 0x504840 sobre Board+0x2DA0 (objeto de clima, não lido). `GetHealth(Point)`
copia o objeto de vida da casa (0x561CE0, depois 0x5AF100) e devolve o campo +4, a vida atual (+0x78 da casa).

## Funções Lua usadas (scripts originais)
`getBuildingHealth(0)` e `(1)` = {50, 50, 0, 0}; demais setores {50, 40, 10, 0}. `getEnvironmentChance` dá 16% de
floresta na grama (Normal), gasta `rand()` em `random_element` só para ácido no setor "acid". `getCrackChance` gasta
um `random_bool(5)`. `getRainChance` dá 20% em grama, neve e ácido. `Mission:IsCracksAllowed` = `not self.BlockCracks`.

## Teste contra o jogo
Duas partidas novas na dificuldade Normal, sem conteúdo avançado:

| Missão | Mapa (achado pelo terreno) | Pontos de `rand()` em sincronia | Casas diferentes | Divergências |
|---|---|---|---|---|
| `Mission_Airstrike` (Mission6) | `any30` | 6 de 7 | 0 | 0 de 65.790 |
| `Mission_Tides` (Mission3) | `hightide18` | 6 de 7 | 0 | 0 de 64.890 |

O primeiro ponto fica fora porque `SelectMap` e o que vem antes ainda não existem no Rust. O nome do mapa não sai pela
Lua: o envoltório de `AddMap` não registrou o nome e nenhuma global de mapa mudou de tabela (motivo não lido). O comparador acha o mapa pelo terreno.

## Ainda não lido
`SelectMap` 0x582A60, `InitializeMission` 0x583130, o objeto de clima 0x504840, quem grava o setor 0x8D1A88 e a conversão
de retorno 0x597F50 (o Rust aceita número e trata o resto como 0).
