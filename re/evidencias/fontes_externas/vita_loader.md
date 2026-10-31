# Fonte externa: Into-The-Breach-Vita (loader de Rinnegatamante)

Repositório sem licença: daqui só se registram FATOS (nomes de símbolos e comportamento descrito), nenhum
código foi copiado. Ele carrega o `libmain.so` ARMv7 da versão Android/Netflix 1.2.92 e substitui funções
pelos nomes exportados — o que confirma que o `libmain.so` exporta símbolos C++ com nome.

## Símbolos do jogo citados pelo loader (`loader/main.c`, função `patch_game`)
| Símbolo | Nome | O que o loader faz | Existe no PC 1.2.88 (RTTI)? |
|---|---|---|---|
| `_ZN4CApp6OnInitEv` | `CApp::OnInit()` | depois dela abre o controle e chama `Settings::SetController` | `CApp` sim |
| `_ZN8Settings13SetControllerEP19_SDL_GameController` | `Settings::SetController(SDL_GameController*)` | liga o suporte a controle (desligado no Android) | — |
| `_ZN8Settings17GetGamepadSettingEv` | `Settings::GetGamepadSetting()` | devolve 1 (ícones de PS4) | — |
| `_ZN4CFPS7GetTimeEv` | `CFPS::GetTime()` (double, ms) | relógio do quadro | — |
| `_ZN6CEvent17IsAppOnBackgroundEv` | `CEvent::IsAppOnBackground()` | devolve 0 | `CEvent` sim |
| `_ZN12CSaveGameDir7LogInfoEj` | `CSaveGameDir::LogInfo(unsigned)` | desligada | `CSaveGameDir` sim |
| `_ZN12SoundControl9TrackDoneEjP25FMOD_STUDIO_EVENTINSTANCEPv` | `SoundControl::TrackDone(...)` | callback de fim de faixa de música | — |
| `_Z13GetAppVersionv`, `_Z18GetNetflixLanguagev`, `_Z17IsLoggedToNetflixv` | funções livres | só da versão Netflix | — |
| `android_fopen`, `SDL_main`, `Java_org_libsdl_app_SDLActivity_nativeInitAssetManager` | plataforma | arquivos e entrada | — |

`CFPS::GetTime` ajuda a entender o relógio do quadro usado em `g_8B6AE0` (Etapa 12).

## Superfície do FMOD Studio usada pelo jogo
O loader lista 24 funções do FMOD; as importações do `Breach.exe` (PC) são 20 e coincidem:
`System::create/initialize/update/release/getBus/loadBankFile`, `Bank::loadSampleData/getEventCount/getEventList`,
`EventDescription::getPath/isOneshot/isStream/createInstance`, `EventInstance::start/stop/release/setCallback/
setParameterValue` (Android: `setParameterByName`), `Bus::setVolume/setMute` (Android também usa
`getCoreSystem`, `System::getVersion`, `mixerSuspend/Resume`).
Consequência para a decisão 0001: o tocador de som em Rust precisa cobrir só isso — carregar bancos, achar
eventos pelo caminho, criar/iniciar/parar instâncias, um parâmetro por nome, volume/mudo por barramento e o
aviso de fim de faixa.

## Arquivos
O loader redireciona `loadBankFile` para `resources/<nome>.bank`: os bancos ficam na pasta `resources`.
Os dados vêm da pasta `assets` do APK.

## Próximo passo possível
O prêmio maior é o próprio `libmain.so` (APK da Netflix, `lib/armeabi-v7a`): com a tabela de símbolos dele,
dá para dar nome às funções que o projeto hoje chama por endereço. Só com o jogo obtido legalmente, no PC do
usuário, sem versionar o arquivo.
