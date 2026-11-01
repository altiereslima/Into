# Migração 1.2.88 → 1.2.93

`Breach.exe` 1.2.93: SHA256 `0f5bdc9098135d4c77ce5bd96614addf371896523ffeaea045b9f9a0d872b910`, carimbo PE 1710319853.
A versão vem da string `1.2.93` dentro do EXE. Os arquivos de `re/evidencias/` fora desta pasta continuam na 1.2.88.

## Arquivos
| Arquivo | Conteúdo | Gerado por |
|---|---|---|
| `lua_api_v2_all.csv`, `lua_api_v2_classes.csv`, `lua_global_values.csv`, `lua_api_v2_progress.json` | inventário da API | `map_lua_api_v2.py` |
| `nomes_android.csv` | nomes do `libmain.so` casados com a 1.2.93 | `match_android_symbols.py` |
| `pares_manuais.csv` | pares lidos no assembly, com a prova | à mão |
| `nomes_manual.csv` | nomes achados direto na 1.2.93, pelos chamados das mesmas funções no `libmain.so`, com a prova | à mão |
| `migracao_enderecos.csv` | tabela antigo → novo, com método e confiança | `migrar_enderecos.py` |
| `regioes_alteradas.csv` | pontos em que o deslocamento muda | `migrar_enderecos.py` |
| `migracao_resumo.txt` | números da última execução | `migrar_enderecos.py` |

## Refazer
```bash
python re/tools/map_lua_api_v2.py <Breach.exe> re/evidencias/v1.2.93/
python re/tools/match_android_symbols.py <Breach.exe> <libmain.so> re/evidencias/v1.2.93/
python re/tools/migrar_enderecos.py <Breach.exe>
python re/tools/map_lua_api_v2.py <Breach.exe> re/evidencias/v1.2.93/
python re/tools/conferir_1293.py <Breach.exe>
python re/tools/check_inventory_v2.py <Breach.exe> <pasta do jogo>
python re/tools/gen_api_manifest.py
python re/tools/gen_nomes_resumo.py
```
`migrar_enderecos.py` reescreve `re/tools/rust_status.json`, por isso o inventário roda de novo depois dele.
`match_android_symbols.py` precisa de `c++filt` no PATH. Sem `llvm-undname`, o inventário reaproveita as assinaturas
legíveis da 1.2.88 quando o nome RTTI é o mesmo.

## Âncoras do inventário
Achadas por estrutura, sem partir dos valores da 1.2.88, e provadas pelo resultado:

| Âncora | 1.2.88 | 1.2.93 | Prova |
|---|---|---|---|
| `class_base::class_base` | 0x006E3F10 | 0x006E4DB0 | `push nome` + chamada para 32 de 32 classes |
| `operator new` | 0x00753C6F | 0x00754AD5 | 274 registradores, o mesmo número da 1.2.88 |
| `operator[]` de `luabind::globals` | 0x00450870 | 0x00450870 | 197 valores iguais aos da 1.2.88 |
| Atribuições locais (int, unsigned, bool, objeto) | 0x006802F0… | +0xEC0 | único deslocamento com 197 de 197 valores |
| Importações `lua_push*` | 0x007D2538… | 0x007D3540… | tabela de importação do PE |

## Pares de endereços
Sinais independentes: mesmo registro luabind, mesma função do Android nas duas versões, e bytes das provas da
Etapa 05 com curingas nos campos de endereço. O inventário vence em conflito. O código mantém a ordem, então um par
fora da maior sequência crescente é erro de casamento; um par só do Android que destoa dos dois vizinhos também.
Com isso, 11 pares do Android foram descartados e 1 conflito foi resolvido pelo inventário.

O erro mais importante: os nomes da 1.2.88 punham `ActiveLua::GetScripts` em 0x00675C70, mas o conteúdo dessa rotina
é só `SetPawn/SetBoard/SetGame(nil)`. A rotina que lê `GetScripts` é o slot 2. O par errado criava uma falsa região
alterada. `nomes_manual.csv` agora tem os dois nomes certos.

## O que `Pawn::Push` mostrou
Lida na 1.2.93 (0x0062DAB0) e no `libmain.so` (`_ZN4Pawn4PushEi`), com a mesma estrutura:
* `DIR_FLIP`: testa +0x91F (Android +0x7F3, `IsFlippable`). `Pawn::OnInit` grava nele o inverso do campo `IgnoreFlip`.
* Outras direções: +0x91E == 0 (`IsGuarding`; `OnInit` grava o campo `Pushable`) faz `SetGuard(false)` em +0x8D2 e sai.
* O resto arma 0,4 s em +0x968 e grava a direção em +0x978.

O Rust usava +0x91E no `DIR_FLIP`, então uma unidade não empurrável nunca teria o ataque virado. Corrigido em
`apply.rs`, com teste. O campo +0x8D2 não foi modelado: na varredura da 1.2.93 só há escritas nesse deslocamento.

## Limites
* Não há bytes da 1.2.88 além das 24 provas. Deslocamento igual prova tamanho igual, não bytes iguais.
* `Pawn::Push` tem o mesmo início até armar o cronômetro (deslocamento 0x151 nas duas versões), mas ficou ao menos
  4 bytes menor depois dele. Sem os bytes antigos, não dá para dizer o que saiu.
* Os scripts da 1.2.88 também não estão aqui, então eles não foram comparados. Os 4 `LOG("No forced end!")` que a
  varredura imprime vêm de `ae_weapons.lua`; o relatório da Etapa 12 não os tem, e a causa não foi apurada.
* O `libmain.so` é a 1.2.92. Ele ajuda a dar nomes, mas não prova nada sobre a 1.2.93 sozinho.
