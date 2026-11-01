# CHECKPOINT — Migração para o Breach.exe 1.2.93

**Data:** 2026-11-01. Etapas anteriores em `docs/historico/`. Método e limites em
`re/evidencias/v1.2.93/LEIA-ME.md`.

## Feito
- **Base do projeto agora é a 1.2.93** (SHA256 `0f5bdc90…b910`, instalação com `resources/`).
- **Inventário da API** gerado na 1.2.93: `map_lua_api_v2.py` aceita as duas versões (`PROFILES`). O manifesto só mudou nos endereços.
- **Tabela de endereços 1.2.88 → 1.2.93** (`migrar_enderecos.py`), sem o EXE antigo: inventário, Android e bytes das provas, conferidos entre si.
- **Nomes do Android** casados direto com a 1.2.93, com o mesmo `libmain.so` de antes.
- **Conferência repetível:** `conferir_1293.py` lê 36 fatos no EXE novo (tempos, tabelas, constantes, slots de vftable).
- **Erro corrigido em `Pawn::Push`:** `DIR_FLIP` depende de `IgnoreFlip` (+0x91F), não de `Pushable` (+0x91E). Vale na 1.2.93 e no Android 1.2.92.
- **Erro corrigido nos nomes:** `ActiveLua::GetScripts` é o slot 2; o slot 1 só faz `SetPawn/SetBoard/SetGame(nil)`.

## O que mudou entre as versões
| Item | Resultado |
|---|---|
| Registros luabind (551), ordem e assinaturas RTTI | idênticos |
| Classes Lua (32) | idênticas |
| Valores globais (197) | idênticos |
| Provas da Etapa 05 com bytes (24) | as 24 achadas, iguais fora os campos de endereço |
| Pontos em que o deslocamento muda | 52, somando 3669 bytes em cerca de 4 MB |
| Maior mudança | carregamento de arquivos e fontes (+3216 bytes); depois salvamento (+400) |

## Números
| Métrica | Antes | Agora |
|---|---|---|
| Versão de referência | 1.2.88 | **1.2.93** |
| Endereços citados no projeto com par na 1.2.93 (342) | — | **342** (312 início de função ou prova direta; 30 pontos internos interpolados) |
| Boot: arquivos ok / ausentes / erros | 137 / 1 / 0 | **137 / 1 / 0** |
| Armas sem erro nem stub (559) | 559 | **559** |
| Nomes reconstruídos / plataforma / stubs (475) | 212 / 13 / 250 | **212 / 13 / 250** |
| Testes | 99 + 2 | **100 + 2** |

## Ainda não provado
- Sem o EXE 1.2.88, deslocamento igual prova tamanho igual, não bytes iguais. Uma mudança do mesmo tamanho passaria despercebida.
- Das 52 regiões, só foram lidas as que contêm rotinas citadas nas regras (`Pawn::Push`, `Pawn::GetType`, `ActiveLua`, `GetPawnCount`). Em `Pawn::Push`, o código muda depois de armar o cronômetro de 0,4 s; o motivo não foi identificado, porque não há os bytes antigos, mas a versão atual segue igual à do Android.
- Provas em `re/evidencias/rotinas_*.md` e endereços nos comentários do Rust continuam na 1.2.88; a tradução está em `v1.2.93/migracao_enderecos.csv`.
- Nomes provisórios no código (`Tile::Push`, `byte_0x91e`…) ainda não foram trocados pelos reais.

## Próximo
Etapa 13, lida direto na 1.2.93: tempo de voo dos projéteis, animações de movimento, efeito de chegada completo; depois o turno inimigo (M6). Gráficos e som (decisão 0001) já têm os arquivos necessários.
