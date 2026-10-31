# Etapa 08 — SkillEffect, listas e Board/Pawn na Lua

Breach.exe v1.2.88. Leitura com `capstone`; script auxiliar de extração descrito abaixo.

## Layout
- `SpaceDamage` = 0x134 bytes. Campos internos usados: +0x34 parâmetro de animação (padrão 2),
  +0x68 imagem de projétil/artilharia, +0x98 tipo (1 artilharia, 2 projétil, 5 laser),
  +0x9C origem (-1,-1), +0xCC..+0xD4 `vector<Point>` caminho, +0xD8 tipo de movimento (3 = corpo a corpo).
- `SkillEffect` (ctor 0x0055D8F0): effect +0x00 e q_effect +0x0C (`vector<SpaceDamage>`), impact_sound +0x30,
  piOrigin +0x48 = Point() (0x80000001), +0x58 = 2, iOwner +0x5C = -1.
- Listas: `index(i)` é 1-based e sem checagem (`[begin + i*8 - 8]`); `DamageList:index/back` e
  `SkillEffect:GetDamage` devolvem **referência** (`SpaceDamage&`).

## Métodos reconstruídos
| Método | VA | Regra |
|---|---|---|
| AddDamage / AddQueuedDamage | 0x00652A50 / 0x00652CB0 | cópia → effect / q_effect |
| AddDelay(f) | 0x00653120 | SpaceDamage(0), fDelay=f, bHide=true |
| AddScript / AddQueuedScript / AddSound | 0x00654220 / 0x00654580 / 0x00654670 | SpaceDamage(0) com sScript / sSound |
| AddAnimation(p,s[,n]) | 0x00654310 / 0x00654400 | SpaceDamage(p,0), sAnimation=s, +0x34=n |
| AddProjectile | 0x00653370 (+ 0x006531D0, 0x006532A0) | imagem; tipo 5 se contém "laser" senão 2; atraso 0 se laser; origem; atraso padrão −2 |
| AddQueuedProjectile | 0x00653740 (+0x00653850) | idem, sem origem, atraso sempre o informado |
| AddArtillery / AddQueuedArtillery | 0x00653640 (+0x006534A0, 0x00653570) / 0x00653920 (+0x00653A10) | imagem, tipo 1, atraso (padrão −2), origem só na imediata |
| AddMelee / AddQueuedMelee | 0x00654B50 (+0x00654AE0) / 0x00654C70 (+0x00654C00) | tipo de movimento 3, origem anexada ao caminho, atraso (padrão −1) |
| AddMove / AddQueuedMove | 0x00653AE0 / 0x006549F0 | caminho com ≤ 1 ponto → false; senão SpaceDamage(0) com caminho e atraso |
| GetDamageCount/GetQueuedCount/GetDamage/GetQueuedDamage | 0x006528E0 / 0x00652E30 / 0x00652900 / 0x00653100 | tamanho; referência 1-based |
| SpaceDamage:IsMovement/MoveStart/MoveEnd | 0x00655F10 / 0x00655F40 / 0x00655F80 | caminho não vazio e tipo≠3 e +0x98==0; início ou (−1,1); último |

Pendentes (stubs): AddBounce, AddLeap, AddCharge, AddBurrow, AddTeleport, AddGrapple, AddBoardShake,
AddVoice, AddQueuedVoice, AddAirstrike, AddReverseAirstrike, AddDropper, AddBurst, AddEmitter, AddQueuedCharge.

## Board / Pawn
`Pawn::IsTeam` 0x00639FD0 (2 → sempre; 8 → +0x10BC==1; 7 → time 6 e +0x10BC==0; 6 → time ≥ 6;
−1 → IsTeam(6) e 0x0063A410(); 4 → mech ou time 4; 5 → não neutro (+0x97C) e jogador, ou time 5; demais → igualdade).
`Board:IsPawnTeam` 0x0056ED50 / 0x0056EDA0: sem unidade → `team == 2`; senão `GetPawn(p)->IsTeam(team)`.
`Board:GetSize` 0x0055DC10: (largura, altura).

## Demonstração
`itb-weapon` executa `GetTargetArea`/`GetSkillEffect` originais (ver `docs/relatorios/armas_etapa08.txt`).
Ex.: `Prime_Lasermech` gera dano 3, 2, 1, 1 ao longo da linha, com o último efeito marcado como laser (tipo 5).
