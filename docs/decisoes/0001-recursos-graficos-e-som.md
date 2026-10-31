# Decisão 0001 — Gráficos e som: ler da instalação e extrair localmente

**Data:** 2026-10-31 · **Estado:** aceita (implementação planejada para depois do turno inimigo, M6)

## Contexto
O motor em Rust já executa os scripts Lua originais lendo-os da instalação do usuário. Faltam gráficos
(`resources/resource.dat`, PNGs e fontes empacotados) e som (`data/*.bank`, FMOD Studio). O usuário quer
também facilitar um port futuro para consoles.

## Decisão
1. **Nada do jogo entra no repositório.** O repositório guarda só o código que sabe abrir os formatos.
2. **Ferramenta `itb-assets`** (Rust): lê a instalação e gera, NA MÁQUINA DO USUÁRIO, uma pasta
   `recursos_extraidos/` com:
   - imagens em PNG e fontes, mais `indice.json` com o caminho original de cada arquivo;
   - sons e músicas decodificados dos `.bank` (Ogg/WAV), mais índice ligando cada evento FMOD
     (ex.: `/props/shield_destroyed`) aos arquivos e às regras do evento.
3. **Motor com duas fontes de recursos e a mesma interface:** (a) direto de `resource.dat` + `.bank`
   (PC); (b) da pasta extraída (base para ports; cada plataforma converte para o seu formato).
4. **Som sem depender do FMOD:** as regras dos eventos (variações, camadas, transições de música) serão
   lidas dos bancos e reimplementadas em Rust, para não exigir licença do FMOD em cada plataforma.
5. **Trabalho local:** a análise e a extração rodam no PC do usuário (ponte com o app Claude e a pasta do
   jogo conectada), sem enviar os arquivos grandes. Os originais nunca são alterados.

## Consequências
- `recursos_extraidos/` fica no `.gitignore`.
- Ports para console: viáveis tecnicamente pela separação regras/gráficos/som/entrada; uso pessoal em
  console desbloqueado. Distribuição pública exigiria licença das plataformas e autorização da Subset Games.
- Os pedidos visuais e sonoros que hoje viram `BoardEvent` ou são ignorados (animações, emissores, sons)
  passam a ser chamadas ao renderizador e ao tocador de som.

## Pendências para começar
Usuário: abrir a tarefa no app Claude do PC, "Link to this computer" e conectar a pasta
`Steam/steamapps/common/Into the Breach`. Primeiro passo técnico: confirmar o formato do `resource.dat`
(provavelmente semelhante ao do FTL) e o conteúdo dos `.bank` (FSB5/Vorbis esperado).
