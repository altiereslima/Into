# RE estática — Breach.exe (v1.2.88) — Mapeamento 03

SHA256 `Breach.exe`: `4bd9e4cc43140244d7def56d440346be836efc5b072dd3e8b0dda3ff31fc84b0`.
Base PE32: `0x00400000`. Os endereços abaixo são VAs sem ASLR. Foram extraídos do binário recebido nas fontes do projeto, não de fontes hipotéticas.

## Funções validadas pela leitura de instruções

| Função registrada na API Lua | Registro de referência | VA do corpo | Evidência e comportamento (não é o mesmo que teste no jogo) |
|---|---|---|---|
| Board.IsValid | `0x00676411` | `0x00561370` | Já etapa 01R; compara X,Y com limites em `this+0x48/0x4C`, retorna AL=0/1 |
| Board.GetTerrain | `0x00676741`, `0x00676766` | `0x00503B60` | `call 0x0055DC30`; lê DWORD `[eax+0x2AE0]` |
| Board.IsBuilding | `0x006767B0`, `0x006767D5` | `0x00567D00` | `call 0x0055DC30`, `cmp DWORD [eax+0x2AE0],1`, `sete al` |
| Board.IsEdge | `0x006772AB` | `0x0056FB20` | Compara `x==0`, `y==0`, `x==width-1`, `y==height-1`; **não testa is_valid** |
| Board.IsFire | `0x00677000` | `0x00567C50` | Virtual `vtable+8` testa localização; depois DWORD por célula `[tile+0x2AEC] != 0` ou sentinela `[this+0x2B48]` |
| Board.IsSmoke | `0x00677025` | `0x005653E0` | `vtable+8` testa localização; BYTE por célula `[tile+0x2B25]`, ou sentinela `[this+0x2B81]` |
| Board.IsVines | `0x0067704A` | `0x005653C0` | `call 0x00560E60`; BYTE `[tile+0x2AA0]`; sentinela é `[this+0x5C+0x2AA0]` |
| Board.IsAcid | `0x00677286` | `0x00567CB0` | Guarda virtual; BYTE `[tile+0x2AF1]`, ou `[this+0x2B4D]` quando inválida |
| Board.IsCracked | `0x006762F1` | `0x00569920` | Guarda virtual; BYTE `[tile+0x2B26]`, ou `[this+0x2B82]` quando inválida |
| Board.GetPawnTeam | `0x006767F7` | `0x00568950` | Chama `0x0059F050` com parâmetro 1; se encontrar pawn qualificado, lê primeiro ponteiro da lista e DWORD `[pawn+0xB0]`; senão retorna `eax=2` |

## Acesso a casa / layout binário original

`0x0055DC30` e `0x00560E60` usam `vtable+0x08` como guarda de coordenadas. Se a guarda é verdadeira:

- Colunas: `this+0x50` aponta para tabela; endereço base da coluna x = `DWORD[table + x*12]`;
- Linha: `0x2BBC * y` acrescido à base de coluna (não é tamanho fixo do tabuleiro Rust);
- Se falhar: endereço da **casa sentinela** `this+0x5C`.

Importante: `vtable+0x08` é uma verificação virtual, não provamos que coincide em todos os subtipos com `Board.IsValid` sem inspeção da vtable. A estrutura `BoardSnapshot` é só uma equivalência semântica de consultas e usa armazenamento linha-a-linha, não essa disposição de memória.

## Pendências com evidências, mas sem reconstrução semântica ainda

- `Board.IsBlocked`: registro `0x00676261` aponta para `0x006E015F`, aparenta ser um stub, **não** a rotina substantiva de colisão.
- `Board.IsPawnSpace`: registro `0x00676436` aponta para thunk virtual `0x004E6260`, que salta para `[vtable+0x50]`; implementação real depende de vtable/instância.
- `Board.GetPawn`: existe como string `0x00833774` e composição de sobrecargas em `0x00675EE0`/`0x00675F80`, mas a função C++ subjacente **não foi comprovadamente identificada** nesta etapa.
- `Board.GetPawnCount`: registro `0x00676BE7` aponta para `0x00566180`; a rotina contém iteração e transformação de strings; não foi reconstruída.
- `Board.GetPawnTeam`: semântica de `0x0059F050(1)` ainda incompleta, o que impede equivalência plena em presença de múltiplos pawns ou pawns filtrados.
- Valores reais da sentinela fora do tabuleiro dependem do estado do jogo; não presumir zeros.

## Como revalidar os trechos

Com GNU binutils (Linux):

```bash
objdump -d -Mintel --start-address=0x55dc30 --stop-address=0x55dc75 Breach.exe
objdump -d -Mintel --start-address=0x503b60 --stop-address=0x503b80 Breach.exe
objdump -d -Mintel --start-address=0x567d00 --stop-address=0x567d20 Breach.exe
objdump -d -Mintel --start-address=0x56fb20 --stop-address=0x56fb50 Breach.exe
objdump -d -Mintel --start-address=0x568950 --stop-address=0x5689b0 Breach.exe
```

O script `tools/map_board_bindings.py` é independente de `pefile`, usa leitura PE32 e um padrão estrito de bytecode de registro; produz a mesma tabela `board_bindings.csv` para a base com SHA correto. A tabela não afirma que qualquer corpo C++ já foi integralmente recuperado.
