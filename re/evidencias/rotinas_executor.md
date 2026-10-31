# Etapa 12 — executor de SkillEffect e relógio

Código: `itb-core/src/executor.rs`. Assembly de `Breach.exe` v1.2.88 (capstone) + angr para leitura.

## Board:AddEffect — 0x0055F7B0 (SkillEffect) / 0x0055F970 (SpaceDamage)
Copia o SkillEffect (0x7C bytes) para o vetor Board+0x2C50 e empilha o cronômetro −1.0 (0xBF800000) em
Board+0x2C5C. A versão com SpaceDamage cria um SkillEffect (0x0055D8F0), faz `AddDamage` e chama a outra.

## Atualização do tabuleiro — 0x00568CF0
`estado = 0x005689F0()`; ocupado = estado ∈ {1, 2, 8}. Para cada lista da fila, em ordem:
* cronômetro > 0: subtrai `g_8B6AE0 × 0.0625` (soma móvel dos últimos quadros, 0x0048F5C9); se ainda > 0, espera;
* cronômetro == −1: espera se ocupado;
* cronômetro == −2: espera se estado == 8;
* lista vazia: sai da fila e o laço continua;
* senão: sai da fila e vai para o executor (WeaponTarget slot 0x0C); só UMA por atualização.

## Executor — 0x005601C0
Para cada `SpaceDamage` da lista (passo 0x134):
1. tipo de movimento (+0xD8) == 3 (corpo a corpo): `GetPawn(caminho[0])` (0x00570E00); sem unidade →
   "COULD NOT FIND UNIT FOR MELEE ATTACK?" e `DamageSpace` direto; com unidade → 0x00632B40 guarda a cópia
   em Pawn+0xC84, arma o cronômetro de 0,15 s (0x3E19999A em +0x968) e vira a unidade para o alvo (+0x978).
2. senão, tipo (+0x98) == 0 → `DamageSpace` (WeaponTarget slot 0).
3. senão → projétil (0x00656140: tipo 4 lançador 0x0055D260, 3/6 ataque aéreo 0x0055CFA0, 1 artilharia
   0x0055D0A0, demais projétil/laser) guardado em Board+0x2D20; o dano é aplicado na chegada.
4. caminho não vazio, tipo de movimento != 3 e tipo 0 → `GetPawn(caminho[0], false)` (0x00567750) e:
   5 escavar 0x00636610, 1 salto 0x00632750, 2 investida 0x006329F0, 4 teleporte (destino em Pawn+0x10B4),
   outros andar 0x00632670. Sem unidade: aviso "Attempted move skill without a target pawn...".
5. `fDelay` (+0xC8) != 0: as entradas seguintes viram uma nova lista (0x005774F0) no FIM da fila com esse
   cronômetro, e o executor termina.

## Unidade — 0x006308A1 (trecho da atualização)
Quando o cronômetro +0x95C termina: se +0xD5C == 3, `DamageSpace` com o golpe guardado em +0xC84 e limpa;
senão `ResolvePush` (0x0062CDD0). `Pawn::Push` arma 0,4 s (0x0062CD51).
