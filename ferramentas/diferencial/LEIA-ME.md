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
* O motor escolhe o mapa e muda o terreno antes de `BaseStart`. `itb-diff` acha o mapa mais parecido, informa as casas
  diferentes e parte do terreno real, para comparar as regras sobre o mesmo tabuleiro. Esse passo não é testado: ver
  `Board::RandomizeTerrain` em `docs/DIVERGENCIAS.md`.
* `rand()`: `coleta.lua` recupera o estado do LCG. Antes do sorteio de casas, o Rust é posto no mesmo estado.
* `consumo`: quantos `rand()` cada `CreatePawn` gasta.

## Cuidados ao estender a coleta
No original, chamar um método que depende de algo ausente derruba o jogo. Ensaie antes no Rust. A saída não tem buffer,
então a última linha de `diff_original.txt` aponta a chamada culpada. Exemplos reais: `CreatePawn` durante a
inicialização, e `MoveEnd` em dano sem movimento.

## O que não é comparado
Aplicação de efeitos ao tabuleiro, turnos, IA, objetivos, vida de casas e prédios (sem método de leitura), interface e som.
