# Mapa de missão: leitura na 1.2.93

Todos os endereços são da 1.2.93. Código: `itb-core/src/map.rs` e `itb-lua/src/map.rs`.

## Arquivos
Os `maps/*.map` são Lua: `nome = {["version"] = 7, ["dimensions"] = Point( 8, 8 ), ["map"] = {...}, ...}`. São 376
mapas, todos 8x8 e versão 7, e a global sempre tem o nome do arquivo.

`Board::LoadMapData` 0x56CF60 executa `maps/maphelper.lua`, lista a pasta `maps`, executa cada arquivo e chama
`AddMap(nome)`. `Board::Load` 0x565860 executa `maps/<nome>.map`, chama `AddMap` e passa a tabela a
`Board::LoadScript` 0x562220. O lado inverso é `Board::SaveScript` 0x563CF0.

## Topo da tabela
| Chave | Padrão | Efeito |
|---|---|---|
| `dimensions` | — | 0x5676B0 grava o tamanho em Board+0x48 e aloca as casas; 0x56A170 liga as vizinhas |
| `name` | — | Board+0x2C1C |
| `enemy_kills` | 0 | Board+0x7504 |
| `map` | — | laço das casas, abaixo |
| `spawns`, `spawn_ids`, `spawn_points` | vazios | unidades em surgimento; vazios nos 376 mapas |
| `zones` | — | Board+0x2C90 |
| `blocked_points`, `blocked_type` | vazios | `Board::SetBlockSpawn` 0x56C0B0 |
| `tags` | — | 0x56C300 |
| `cave_tiles`, `pod`, `secret`, `rain`, `rain_type`, `pawn_count`, `teleports`, `tele_history` | — | jogos salvos, não nos mapas |

`version` só é gravada por `SaveScript`.

## Laço das casas, na ordem
| Chave | Padrão | Efeito |
|---|---|---|
| `loc` | — | casa; coordenada inválida cai na sentinela (0x561CE0) |
| `pawn` | "" | guardado até o fim do laço |
| `terrain` | 0 | 0x5A0270: limpa a lista de unidades, chama `SetTerrain` e zera escudo, +0x2AA0/+0x2AA1, fogo, cápsula, ácido, fumaça, rachadura e item |
| `populated` | 0 | +0x2710 = valor == 1 |
| `fire` | 0 | +0x2AEC, gravação direta |
| `cracked`, `spikes`, `vines`, `smoke`, `looper`, `frozen` | falso, -1 em `looper` | ausentes nos mapas |
| `poison` | 0 | `SetAcid(valor == 1)` |
| `lava` | falso | +0x2AC8; se verdadeiro, `SetTerrain(3)` e fogo 0 |
| `item`, `used_item`, `unique`, `pod`, `secret`, `people1`, `people2` | vazio ou 0 | ausentes nos mapas |
| `custom` | "" | +0x27B4 |
| `health_max` | 2 | máximo |
| `health_min` | `health_max` | vida atual = min(valor, máximo) |
| (fim) | — | 0x5AF190 grava a vida; `pawn` não vazio chama 0x56EC00 |
| `grappled` | 0 | ausente nos mapas |

`team` aparece em 6 casas, é gravada por `SaveScript`, mas `LoadScript` não a lê.

A chave `health_min` é a vida atual. `SaveScript` grava a vida atual nela.

## Ajuste de vida (0x5AF190)
Grava vida atual em +0x78 e máximo em +0x7C. Depois: entulho, atual = min(máximo, 0); montanha, máximo 2 e atual em
0..2; prédio único (+0x27AC != 0), máximo 1 e atual em 0..1.

## Casas que o mapa não lista
`BoardSpace::BoardSpace` 0x599D70: `Health(4)` (0x4E0CE0 grava atual e máximo), terreno 0, +0x2710 = 1, +0x2BA4 = -1.
Por isso uma casa não listada tem vida 4/4, e uma casa listada sem `health_max`, 2/2.

## `Board:GetZone` (0x56D960 → 0x56DEA0)
Devolve cópia da lista em Board+0x2C90. Se a zona não existe, lista vazia, com uma exceção: `"enemy"` gera as 3 últimas
colunas, linhas 2 a 5, com a coluna no laço externo.

## Conferência
Totais dos 376 mapas, contados à parte por expressão regular no texto dos arquivos e comparados com o tabuleiro em Rust:

| Item | Texto | Rust |
|---|---|---|
| Casas listadas | 8915 | 8915 |
| Prédios | 2518 | 2518 |
| Montanhas | 2393 | 2393 |
| Água | 2295 | 2295 |
| Areia | 440 | 440 |
| Gelo | 329 | 329 |
| Buracos | 255 | 255 |
| Florestas | 99 | 99 |
| Entulho | 63 | 63 |
| Lava | 84 | 84 |
| Ácido | 211 | 211 |
| `populated` | 2518 | 2518 |

Isso prova a leitura dos arquivos e a aplicação do terreno, não que o tabuleiro seja igual ao do jogo em execução.

## O que falta para a missão
* `Board::SetBuildingHP`: os prédios vêm dos mapas com vida 4, valor que deve ser ajustado depois. A rotina não foi lida.
* `BoardPlayer::SelectMap` e o resto do início de missão.
* A chamada 0x56C300 (`tags`) e o restante de 0x5A0270 depois de 0x5A0300 não foram lidos por inteiro.
