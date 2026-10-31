# Arquitetura — como recriar o Into the Breach em Rust

## A decisão (Etapa 06)

As etapas 01–05 tratavam o projeto como "decompilar o `Breach.exe` função por função para Rust".
Esse caminho tem três problemas:

1. **Tamanho errado.** O `.text` tem ~4 MB de código C++. Boa parte é biblioteca (luabind, boost,
   STL, CRT, SDL, FMOD, Galaxy). Traduzir tudo seria anos de trabalho e não deixaria nada jogável no caminho.
2. **A maior parte do jogo nem está no EXE.** Armas, missões, unidades, IA, ilhas, pilotos,
   diálogos e regras de pontuação estão em **~150 scripts Lua** que vêm com o jogo. O EXE é um
   *motor*: tabuleiro, dano, movimento, renderização, áudio, salvamento, UI — e a ponte para a Lua.
3. **Distribuição.** Código traduzido do EXE é obra derivada. Um motor novo que lê os arquivos
   do usuário é o modelo usado por OpenMW, OpenRCT2 e devilutionX.

**Nova estratégia: reimplementar o motor em Rust e executar os scripts ORIGINAIS sem alterá-los.**
O jogador aponta para a própria instalação do Into the Breach; o repositório nunca contém
arquivos do jogo.

O trabalho de engenharia reversa das etapas anteriores **não foi perdido, ganhou função**: o
inventário fechado da API Lua (32 classes, 343 métodos, 39 funções globais, 111 propriedades,
58 construtores, 190 constantes) virou a **especificação** do que o motor Rust precisa oferecer,
e a análise de cada rotina continua sendo como garantimos que o comportamento é idêntico.

## Camadas

```
┌──────────────────────────────────────────────────────────────┐
│  scripts/*.lua ORIGINAIS (da instalação do usuário)          │
├──────────────────────────────────────────────────────────────┤
│  itb-lua   host Lua 5.1 (mlua, Lua 5.1 embutida)             │
│            • api_manifest.rs  ← gerado do EXE (re/tools)     │
│            • classes/funções reais ou stubs que registram uso│
│            • itb-boot: carrega os scripts e mede o que falta │
├──────────────────────────────────────────────────────────────┤
│  itb-core  regras puras, sem IO: tabuleiro, unidades, dano,  │
│            Point/Rect/Color, rand() MSVC, escalas de tela... │
├──────────────────────────────────────────────────────────────┤
│  (próximas) itb-assets  leitura de resource.dat, mapas, fontes│
│             itb-render  SDL2/wgpu   itb-audio  itb-app       │
└──────────────────────────────────────────────────────────────┘
        re/  ferramentas Python + provas (só leitura do EXE)
```

- **`itb-core`** não conhece Lua nem SDL: recebe dados, devolve resultados. É onde ficam as
  rotinas reconstruídas e seus testes. Pode ser usado por um simulador sem gráficos.
- **`itb-lua`** liga a API Lua ao núcleo. Toda classe/função do manifesto existe desde o início:
  as não implementadas são *stubs* que devolvem `nil` e contam chamadas. Assim o boot sempre
  roda e o relatório mostra, por uso real, o que implementar em seguida.
- **`re/`** guarda as ferramentas de análise e as provas (endereços, desmontagens, CSVs). O
  manifesto Rust é regenerado a partir delas; o CI falha se divergirem.

## Princípio de fidelidade

Cada comportamento implementado carrega o endereço da rotina original e um teste. Quando uma
decisão do host não vem do EXE (ex.: tamanho da janela), ela é marcada como `Status::Host`.
Divergências conhecidas ficam em `docs/DIVERGENCIAS.md`. Exemplos de detalhes que só a análise do EXE revelou:

- `lua_tointeger` do `lua5.1.dll` original usa `fistp` → **arredonda ao par** (`Point(2.5, 3.5)` = (2, 4)),
  enquanto uma Lua compilada em x86-64 truncaria.
- `Point()` sem argumentos vale `(-2147483647, -2147483647)`; `Rect2D()` vale `(-1,-1,0,0)`.
- `random_int`/`random_bool` usam o `rand()` da CRT MSVC (LCG 214013/2531011): com a mesma
  semente, o Rust gera a mesma sequência do jogo.

## Como medir progresso

1. **Boot**: quantos scripts originais carregam sem erro (`itb-boot`).
2. **Cobertura da API**: nomes `Reconstructed` / `Host` / `Stub` (475 nomes no total).
3. **Uso real**: quais stubs os scripts chamam, e quantas vezes — a fila de prioridade.
4. **Marcos jogáveis** (ROADMAP): menu → missão carregada → turno inimigo → missão completa.
