# Etapa 09 — passagem, alcance e caminho simples

Breach.exe v1.2.88. Funções decompiladas com **angr** (`re/tools`: ver CHECKPOINT) e conferidas no assembly
nos pontos onde o decompilador errou (ex.: um `continue` inexistente no laço do alcance).

| Rotina | VA | Regra |
|---|---|---|
| Vetores de direção | 0x008CF9AC (init 0x00403030) | UP(0,−1), RIGHT(1,0), DOWN(0,1), LEFT(−1,0) |
| Perpendiculares | 0x004B2C70 | dir 0/2 → [1,3]; dir 1/3 → [2,0] |
| `Tile::GetPawnTeam` | 0x0059F020 | `HasPawn(true)` ? time do 1º item : 2 |
| `Tile::IsPassable(prof)` | 0x0059F260 | perfil = prof & 0xF, time = prof >> 4. 7/1/6/5 → sempre; 4 → bloqueia terreno 1/4/9; 3 → bloqueia 1/4 e casa ocupada; demais → ocupada por outro time bloqueia; terreno 1, estrutura (`IsNonGridStructure`), água (salvo +0x23AC e +0x23CC<15, ou massivo), 4 e 9 bloqueiam |
| `Board::IsFree` (slot 0x14) | 0x00561300 | fora dos limites → false; senão `IsPassable` |
| `GetSimpleReachable` | 0x00573160 → 0x004CEC60 | raios por direção sobre casas livres p/ projétil, até o alcance; inclui o obstáculo; modo "cantos" com as peculiaridades documentadas em `itb-core/src/pathing.rs` |
| `GetSimplePath` | WeaponTarget 0x80 → 0x005731F0 → 0x004CD680 | para cada direção inicial: reto enquanto aproxima, depois vira para `GetDirection(alvo−atual)`; casas livres p/ projétil (alvo pode estar ocupado); 1ª tentativa que chega vence; senão vazio |
| `Pawn::GetPathProf` | 0x0062F700 | `(time<<4) | perfil`: teleporte 5 (se vivo) > saltador 6 > escavador 7 > voador 1 > Road_Runner 4 > massivo 2 > chão 0 |

## Efeitos de SkillEffect adicionados
AddBounce 0x00654EC0 (`sScript = "Board:Bounce(Point( x, y ),n)"`), AddBoardShake 0x00654D20 (`"Board:StartShake(%f)"`),
AddBurst 0x00652E50 (`"Board:AddBurst(Point( x, y ),\"anim\",n)"`), AddCharge/AddQueuedCharge/AddBurrow
(0x00653E90/0x00654960/0x00653E00: AddMove + tipo 2/2/5), AddLeap 0x00653BD0 (marca `advanced/combat/throw_<dir>.png`
+ movimento tipo 1), AddTeleport 0x00654760 (movimento tipo 4 + duas marcas de brilho), AddGrapple 0x00652D30,
global `SoundEffect` 0x00656480.

## Continuação
`GetReachable` e `GetPath` foram reconstruídos na Etapa 10: ver `rotinas_alcance.md`.
