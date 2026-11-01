# Teste diferencial: jogo original x motor Rust

O mesmo `coleta.lua` roda nos dois lados, e `itb-diff` compara as saídas, linha a linha. Nada do jogo entra no
repositório: a cópia fica fora dele, e `diff_original.txt` fica dentro da cópia.

## Rodar
Sem missão, sem cliques (constantes, tipos de valor, `rand()` e as unidades):

```bash
python ferramentas/diferencial/preparar.py "<jogo>" "<cópia>" --modo boot
python ferramentas/diferencial/rodar.py "<cópia>" --tempo 60
cargo run -p itb-lua --bin itb-diff -- "<jogo>" "<cópia>/diff_original.txt"
```

Com tabuleiro, use `--modo missao` e `--tempo 600`. No jogo:

1. Aparece a tela `Something went wrong in Turret_Base::GetClass`. É esperado: uma unidade de `pawns.lua` lista essa
   arma, que não existe, e a coleta cria todos os tipos. Pressione uma tecla.
2. Novo jogo, Iniciar, escolha Normal, Iniciar.
3. Clique na ilha disponível. A coleta roda nesse momento, e o jogo fecha sozinho.

## Dados do usuário
O jogo grava em `Documents/My Games/Into The Breach` mesmo rodando da cópia, e começar uma partida cria um `saveData.lua`.
`rodar.py` guarda essa pasta, roda o jogo e devolve exatamente o que havia, apagando o que apareceu. Não use o
jogo para jogar enquanto ele estiver rodando: o progresso seria descartado.

## Como lê o resultado
* Preparação da missão: o jogo dá vida aos prédios e sorteia terreno e chuva antes de `BaseStart`. `coleta.lua` grava
  o que o motor passa a `getBuildingHealth`, `getEnvironmentChance`, `getCrackChance`, `getRainChance` e
  `CallMethod(..., "IsCracksAllowed")` (seção `preparo`). `itb-diff` escolhe o mapa pelo terreno (o nome não sai pela
  Lua), cria a missão em `GAME.Missions` e roda `Host::load_mission`. O terreno e a vida finais (`casa|x,y.vida`) são
  comparados com o jogo. Saídas antigas, sem `preparo`, ainda usam o terreno real copiado.
* `rand()`: `coleta.lua` recupera o estado do LCG. Antes de cada chamada da preparação, o original grava 4 valores
  (`sincronia_preparo`); o Rust confere se já estava naquele estado, informa no cabeçalho os pontos fora de sincronia e
  volta ao estado do original. O primeiro ponto fica fora porque `SelectMap` não existe no Rust. Antes do sorteio de
  casas, o Rust é posto no mesmo estado.
* `consumo`: quantos `rand()` cada `CreatePawn` gasta.

## Cuidados ao estender a coleta
No original, chamar um método que depende de algo ausente derruba o jogo. Ensaie antes no Rust. A saída não tem buffer,
então a última linha de `diff_original.txt` aponta a chamada culpada. Exemplos reais: `CreatePawn` durante a
inicialização, e `MoveEnd` em dano sem movimento.

## O que não é comparado
Aplicação de efeitos ao tabuleiro, turnos, IA, objetivos, a escolha do mapa, a chuva (sem método de leitura), interface e som.
