# Nomes das funções do PC vindos do Android (libmain.so 1.2.92)

Gerado por `re/tools/match_android_symbols.py` + `nomes_manual.csv` (`re/tools/gen_nomes_resumo.py`).
O `libmain.so` fica no PC do usuário, fora do repositório; aqui só há nomes e endereços.
A coluna 1.2.93 vem de `v1.2.93/migracao_enderecos.csv`.

* casamentos automáticos: 1192 ({'texto': 516, 'vtable': 373, 'lua': 259, 'vizinhos': 40, 'lua~': 4}); manuais: 40
* endereços de código citados no projeto: 540; com nome agora: 333

## Validação
Cada sinal foi conferido contra os outros: vftable × Lua 8/9; texto × Lua/vftable 62/65;
vizinhos (deixando cada casamento de fora) 68/69. Alinhar a ordem das chamadas falhou (0/72) e não é usado.
Confiança `media` = sinal mais fraco (1 string, nome Lua aproximado, vizinhos ou leitura manual).

## Achados
* A casa do tabuleiro é `BoardSpace`; a base de busca é `GridSearchable` (Dijkstra = `CalculateDijkstraMap`, A* = `GetAstarPath`).
* `Pawn::IsMassive()` existe; o byte +0x99D do PC (água não afoga, gelo danificado não mata) cumpre esse papel (inlinado no PC).
* Fila de efeitos: `Board::AddEffect` → `StackEffect` → `UpdateStackedEffects` → `ApplyEffect` (executor).

## Funções citadas no projeto

| PC 1.2.88 | PC 1.2.93 | Nome no Android | Como | Confiança |
|---|---|---|---|---|
| 0x0044AC60 | 0x0044AC60 | `LuaEnv::BindUniversalDefinitions()` | texto | alta |
| 0x0044D1A0 | 0x0044D1A0 | `LuaEnv::RefreshScripts()` | texto | media |
| 0x0046C4E0 | 0x0046C4E0 | `GL_Color::WithAlpha(float) const` | lua | alta |
| 0x0046C510 | 0x0046C510 | `GL_Color::Normalize() const` | lua | alta |
| 0x004BC350 | 0x004BD130 | `Point::Manhattan(Point) const` | lua | alta |
| 0x004BC380 | 0x004BD160 | `Point::GetString()` | lua | alta |
| 0x004BC510 | 0x004BD2F0 | `Point::GetLuaString()` | lua | alta |
| 0x004CD680 | 0x004CE460 | `GridSearchable::GetSingleCornerPath(Point, Point, int)` | manual | media |
| 0x004CDA00 | 0x004CE7E0 | `GridSearchable::GetReachablePoints(Point, int, int)` | manual | alta |
| 0x004CDCD0 | 0x004CEAB0 | `GridSearchable::GetDistance(Point, Point, int)` | lua | alta |
| 0x004CDE60 | 0x004CEC40 | `GridSearchable::CalculateDijkstraMap(Point, int, int, bool)` | manual | alta |
| 0x004CE630 | 0x004CF410 | `GridSearchable::GetAstarPath(Point, Point, int)` | manual | alta |
| 0x004D48D0 | 0x004D56A0 | `BoardManager::IsCursor()` | vtable | alta |
| 0x004DFFC0 | 0x004E0E80 | `ValueBar::AnimatedModify(int)` | manual | media |
| 0x004E01F0 | 0x004E10B0 | `ValueBar::GetValue() const` | lua | alta |
| 0x004E0200 | 0x004E10C0 | `ValueBar::GetMax()` | lua | alta |
| 0x004E6260 | 0x004E7140 | `CApp::OnRequestExit()` | vtable | alta |
| 0x004F6410 | 0x004F7300 | `Pawn::GetHealth()` | lua | alta |
| 0x00503B60 | 0x00504A50 | `Board::GetTerrain(Point)` | lua | alta |
| 0x0055D260 | 0x0055E0E0 | `PylonAnimation::PylonAnimation(SpaceDamage, WeaponTarget*, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | texto | alta |
| 0x0055DB50 | 0x0055E9D0 | `GameMap::AddEvent(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0055DB70 | 0x0055E9F0 | `SkillManager::IsSelected()` | lua | alta |
| 0x0055DC10 | 0x0055EA90 | `GridSearchable::GetSize() const` | manual | alta |
| 0x0055F030 | 0x0055FEB0 | `Board::IsBlocked(Point, int)` | lua | alta |
| 0x0055F0D0 | 0x0055FF50 | `Board::IsTerrain(Point, int)` | lua | alta |
| 0x0055F130 | 0x0055FFB0 | `Board::GetHealth(Point)` | lua | alta |
| 0x0055F190 | 0x00560010 | `Board::SetHealth(Point, int, int)` | lua | alta |
| 0x0055F200 | 0x00560080 | `Board::DamageSpace(SpaceDamage)` | lua | alta |
| 0x0055F5B0 | 0x00560430 | `Board::DamageSpace(Point, int)` | lua | alta |
| 0x0055F680 | 0x00560500 | `Board::PushDamage(Point, int)` | manual | alta |
| 0x0055F7B0 | 0x00560630 | `Board::AddEffect(SkillEffect)` | lua | alta |
| 0x0055F970 | 0x005607F0 | `Board::AddEffect(SpaceDamage)` | lua | alta |
| 0x005601C0 | 0x00561040 | `Board::ApplyEffect(SkillEffect, int)` | texto | alta |
| 0x00560EA0 | 0x00561D20 | `Board::RemovePawn(Pawn*)` | lua | alta |
| 0x00561080 | 0x00561F00 | `Board::RemovePawn(Point)` | lua | alta |
| 0x00561280 | 0x00562100 | `Board::GetLeaveCost(int, int, int)` | manual | alta |
| 0x005612A0 | 0x00562120 | `Board::GetEnterCost(int, int, int)` | manual | alta |
| 0x00561300 | 0x00562180 | `Board::IsEnterable(int, int, int)` | manual | alta |
| 0x00561370 | 0x005621F0 | `Board::IsValid(Point)` | lua | alta |
| 0x00564D20 | 0x00565BA0 | `Board::MarkFlashing(Point, bool)` | lua | alta |
| 0x00564EC0 | 0x00565D40 | `Board::ClearSpace(Point)` | lua | alta |
| 0x00565060 | 0x00565EE0 | `Board::SetTerrain(Point, int)` | lua | alta |
| 0x00565080 | 0x00565F00 | `Board::SetCustomTile(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00565140 | 0x00565FC0 | `Board::GetCustomTile(Point)` | lua | alta |
| 0x005651E0 | 0x00566060 | `Board::SetSmoke(Point, bool, bool)` | lua | alta |
| 0x00565240 | 0x005660C0 | `Board::AddAlert(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x005652D0 | 0x00566150 | `Board::SetAcid(Point, bool)` | lua | alta |
| 0x005653C0 | 0x00566240 | `Board::IsVines(Point)` | lua | alta |
| 0x005653E0 | 0x00566260 | `Board::IsSmoke(Point)` | lua | alta |
| 0x00565490 | 0x00566310 | `Board::GetPawns(int)` | lua | alta |
| 0x00565570 | 0x005663F0 | `Board::GetBuildingsLua()` | lua~ | media |
| 0x00566170 | 0x00566FF0 | `Board::GetEnemyCount()` | lua | alta |
| 0x00567700 | 0x00568570 | `Board::GetPawn(int)` | lua | alta |
| 0x00567750 | 0x005685C0 | `Board::GetPawn(Point, bool)` | lua | alta |
| 0x00567840 | 0x005686B0 | `Board::IsPawnAlive(int)` | lua | alta |
| 0x00567970 | 0x005687E0 | `Board::GetPawnSpace(int)` | lua | alta |
| 0x00567A10 | 0x00568880 | `Board::SetTerrainIcon(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00567AD0 | 0x00568940 | `Board::RandomizeTerrain(int, int)` | lua | alta |
| 0x00567C50 | 0x00568AC0 | `Board::IsFire(Point)` | lua | alta |
| 0x00567CB0 | 0x00568B20 | `Board::IsAcid(Point)` | lua | alta |
| 0x00567D00 | 0x00568B70 | `Board::IsBuilding(Point)` | lua | alta |
| 0x00567E30 | 0x00568CA0 | `Board::SpawnPawn(Pawn*, Point)` | lua | alta |
| 0x00568250 | 0x005690C0 | `Board::SpawnPawn(Pawn*)` | lua | alta |
| 0x005686D0 | 0x00569540 | `Board::SpawnQueued()` | lua | alta |
| 0x00568860 | 0x005696D0 | `Board::IsCrackable(Point)` | lua | alta |
| 0x00568950 | 0x005697C0 | `Board::GetPawnTeam(Point)` | lua | alta |
| 0x005689C0 | 0x00569830 | `Board::GetPawnTeam(int, int)` | lua | alta |
| 0x005689E0 | 0x00569850 | `Board::IsBusy()` | lua | alta |
| 0x005689F0 | 0x00569860 | `Board::GetBusyState()` | lua | alta |
| 0x00568CF0 | 0x00569B60 | `Board::UpdateStackedEffects()` | texto | alta |
| 0x00569920 | 0x0056A790 | `Board::IsCracked(Point)` | lua | alta |
| 0x00569970 | 0x0056A7E0 | `Board::SetCracked(Point, bool)` | lua | alta |
| 0x0056B240 | 0x0056C0B0 | `Board::SetBlockSpawn(Point, int)` | lua~ | media |
| 0x0056B270 | 0x0056C0E0 | `Board::ClearBlockSpawns()` | lua | alta |
| 0x0056B520 | 0x0056C390 | `Board::AddTeleport(Point, Point)` | lua | alta |
| 0x0056BE90 | 0x0056CD00 | `Board::SetLava(Point, bool)` | lua | alta |
| 0x0056BED0 | 0x0056CD40 | `Board::GetDeployLocScore(Point)` | lua | alta |
| 0x0056BFC0 | 0x0056CE30 | `Board::GetTurn()` | lua | alta |
| 0x0056BFD0 | 0x0056CE40 | `Board::IsSafe(Point)` | lua | alta |
| 0x0056C710 | 0x0056D580 | `Board::SetWeather(int, int, Point, Point, float)` | lua | alta |
| 0x0056C750 | 0x0056D5C0 | `Board::StopWeather()` | lua | alta |
| 0x0056C780 | 0x0056D5F0 | `Board::SetFrozen(Point, bool)` | lua | alta |
| 0x0056C7E0 | 0x0056D650 | `Board::IsFrozen(Point)` | lua | alta |
| 0x0056CAF0 | 0x0056D960 | `Board::GetZone(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0056D540 | 0x0056E3B0 | `Board::Bounce(Point, int)` | lua | alta |
| 0x0056D5B0 | 0x0056E420 | `Board::IsSpawning(Point)` | lua | alta |
| 0x0056D860 | 0x0056E6D0 | `Board::IsTargeted(Point)` | lua | alta |
| 0x0056D930 | 0x0056E7A0 | `Board::Ping(Point, GL_Color)` | lua | alta |
| 0x0056D9C0 | 0x0056E830 | `Board::AddPawn(Pawn*, Point)` | lua | alta |
| 0x0056DE50 | 0x0056ECC0 | `Board::AddPawn(Pawn*)` | lua | alta |
| 0x0056DFD0 | 0x0056EE40 | `Board::AddPawn(Pawn*, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | texto | media |
| 0x0056E590 | 0x0056F400 | `Board::Fade(int)` | lua | alta |
| 0x0056EA40 | 0x0056F8B0 | `Board::Slide(int)` | lua | alta |
| 0x0056ED50 | 0x0056FBC0 | `Board::IsPawnTeam(Point, int)` | lua | alta |
| 0x0056EDA0 | 0x0056FC10 | `Board::IsPawnTeam(int, int, int)` | lua | alta |
| 0x0056EE50 | 0x0056FCC0 | `Board::IsPawnSpace(Point)` | lua | alta |
| 0x0056EE70 | 0x0056FCE0 | `Board::IsPawnSpace(Point, bool)` | lua | alta |
| 0x0056F170 | 0x0056FFE0 | `Board::SetPopulated(bool, Point)` | lua | alta |
| 0x0056F940 | 0x005707B0 | `Board::GetBuildingCount(int)` | lua | alta |
| 0x0056FB20 | 0x00570990 | `Board::IsEdge(Point)` | lua | alta |
| 0x00570430 | 0x005712A0 | `Board::LockBomb()` | lua | alta |
| 0x00570660 | 0x005714D0 | `Board::MarkSpaceColor(Point, GL_Color)` | lua | alta |
| 0x005706C0 | 0x00571530 | `Board::MarkSpaceSimpleColor(Point, GL_Color)` | lua | alta |
| 0x005706E0 | 0x00571550 | `Board::MarkSpaceDamage(SpaceDamage)` | lua | alta |
| 0x005707C0 | 0x00571630 | `Board::GetPawnCount(int)` | lua | alta |
| 0x00570990 | 0x00571800 | `Board::MarkSpaceImage(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, GL_Color)` | lua | alta |
| 0x00570E00 | 0x00571C70 | `Board::GetPawn(Point)` | lua | alta |
| 0x00571680 | 0x005724F0 | `Board::IsPod(Point)` | lua | alta |
| 0x005716F0 | 0x00572560 | `Board::IsItem(Point)` | lua | alta |
| 0x00571750 | 0x005725C0 | `Board::IsDangerousItem(Point)` | lua | alta |
| 0x005717A0 | 0x00572610 | `Board::IsDangerous(Point)` | lua | alta |
| 0x00571870 | 0x005726E0 | `Board::GetItem(Point)` | lua | alta |
| 0x00571930 | 0x005727A0 | `Board::SetDangerous(Point)` | lua | alta |
| 0x00571950 | 0x005727C0 | `Board::SetItem(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00571B20 | 0x00572990 | `Board::IsPushBlocked(Point, int)` | manual | alta |
| 0x005723D0 | 0x00573240 | `Board::IsEnvironmentDanger(Point)` | lua | alta |
| 0x00572680 | 0x005734F0 | `Board::GetRandomBuilding(bool)` | lua | alta |
| 0x005728D0 | 0x00573740 | `Board::GetInfectedCount()` | lua | alta |
| 0x00572A50 | 0x005738C0 | `Board::GetAnotherPlayerPawn(int, int)` | lua | alta |
| 0x00572B50 | 0x005739C0 | `Board::GetMechDamage()` | lua | alta |
| 0x00572D90 | 0x00573C00 | `Board::GetDistanceToPawn(Point, int)` | lua | alta |
| 0x00573030 | 0x00573EA0 | `Board::GetDistanceToBuilding(Point)` | lua | alta |
| 0x005730B0 | 0x00573F20 | `Board::AddAnimation(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00573160 | 0x00573FD0 | `Board::GetSimpleReachable(Point, int, bool)` | lua | alta |
| 0x005731F0 | 0x00574060 | `Board::GetSimplePath(Point, Point)` | lua | alta |
| 0x00573280 | 0x005740F0 | `Board::GetReachablePointList(Point, int, int)` | lua~ | media |
| 0x005733D0 | 0x00574240 | `Board::GetPath(Point, Point, int)` | lua | alta |
| 0x00573490 | 0x00574300 | `Board::AddBurst(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00573540 | 0x005743B0 | `Board::IsWall(Point, int, int)` | lua | alta |
| 0x005735E0 | 0x00574450 | `Board::SetWall(bool, Point, int)` | lua | alta |
| 0x00573680 | 0x005744F0 | `Board::AddUniqueBuilding(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00573820 | 0x00574690 | `Board::IsUniqueBuilding(Point)` | lua | alta |
| 0x00573950 | 0x005747C0 | `Board::IsDamaged(Point)` | lua | alta |
| 0x005739E0 | 0x00574850 | `Board::IsSkillDeadly(SpaceDamage, Pawn*)` | manual | alta |
| 0x00573B80 | 0x005749F0 | `Board::StartMechTravel()` | lua | alta |
| 0x00573EB0 | 0x00574D20 | `Board::StartPopEvent(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0057EAC0 | 0x0057F930 | `SkillManager::GetArmedWeaponId()` | lua | alta |
| 0x0057EAD0 | 0x0057F940 | `SkillManager::IsWeaponArmed()` | lua | alta |
| 0x0059C5C0 | 0x0059D420 | `BoardSpace::CheckTerrainDangers()` | texto | alta |
| 0x0059E8A0 | 0x0059F700 | `BoardSpace::SetFrozen(bool)` | vizinhos | media |
| 0x0059EBE0 | 0x0059FA40 | `BoardSpace::SetForceField()` | texto | alta |
| 0x0059EE20 | 0x0059FC80 | `BoardSpace::RemoveForceField()` | manual | alta |
| 0x0059F050 | 0x0059FEB0 | `BoardSpace::IsPawnSpace(bool)` | manual | alta |
| 0x0059F260 | 0x005A00C0 | `BoardSpace::IsWalkable(int)` | manual | media |
| 0x0059F3A0 | 0x005A0200 | `BoardSpace::GetEnterPenalty(int)` | manual | media |
| 0x0059F7C0 | 0x005A0620 | `BoardSpace::IsWall(int)` | manual | alta |
| 0x005A03E0 | 0x005A1240 | `BoardSpace::ClearItem()` | texto | alta |
| 0x005A0CB0 | 0x005A1B10 | `BoardSpace::SetTerrain(int)` | texto | alta |
| 0x005A10B0 | 0x005A1F10 | `BoardSpace::AddBuilding()` | manual | alta |
| 0x005AAA10 | 0x005AB870 | `BoardSpace::IsPushBlocked(int)` | manual | alta |
| 0x005AABD0 | 0x005ABA30 | `BoardSpace::SetAcid(bool)` | manual | alta |
| 0x005AAC40 | 0x005ABAA0 | `BoardSpace::SetCracked(bool)` | texto | media |
| 0x005AAE10 | 0x005ABC70 | `BoardSpace::DamageTerrain(int)` | texto | media |
| 0x005AB580 | 0x005AC3E0 | `BoardSpace::SetFire(bool)` | texto | alta |
| 0x005AB740 | 0x005AC5A0 | `BoardSpace::SetSmoke(bool, bool)` | texto | media |
| 0x005ABA90 | 0x005AC8F0 | `BoardSpace::DamageSpace(SpaceDamage, int)` | texto | alta |
| 0x005ADBB0 | 0x005AEA10 | `BoardSpace::PushSpace(int)` | manual | alta |
| 0x005AE490 | 0x005AF2F0 | `BoardSpace::GetPawn(bool)` | manual | alta |
| 0x005AE510 | 0x005AF370 | `BoardSpace::IsTerrainBlocking(int)` | manual | alta |
| 0x005AE5D0 | 0x005AF430 | `BoardSpace::IsTerrain(int)` | manual | alta |
| 0x005AE6B0 | 0x005AF510 | `BoardSpace::IsCrackable()` | manual | alta |
| 0x005B99F0 | 0x005BA850 | `RepObj::Failed()` | lua | alta |
| 0x005B9B20 | 0x005BA980 | `RepObj::GetLuaString()` | lua | alta |
| 0x005C4B90 | 0x005C59F0 | `PowerBar::Damage(int)` | texto | media |
| 0x005CEA20 | 0x005CF880 | `ChoiceEvent::SetEffect(Effect)` | lua | alta |
| 0x005CEA80 | 0x005CF8E0 | `ChoiceEvent::SetStartText(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x005CFB30 | 0x005D0990 | `ChoiceEvent::AddChoice(Choice)` | lua | alta |
| 0x005CFCA0 | 0x005D0B00 | `ChoiceEvent::GetText()` | lua | alta |
| 0x005CFCC0 | 0x005D0B20 | `ChoiceEvent::SetHiddenEffects(bool)` | lua | alta |
| 0x005CFCD0 | 0x005D0B30 | `ChoiceEvent::SetUniqueMech(bool)` | lua | alta |
| 0x005F04B0 | 0x005F1340 | `Effect::AddReward(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x006088E0 | 0x00609770 | `GameMap::AddWeapon(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x006089B0 | 0x00609840 | `GameMap::RemoveItem(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00608A80 | 0x00609910 | `GameMap::AddPilot(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00609280 | 0x0060A110 | `GameMap::IsIslandUnlocked(int)` | lua | alta |
| 0x0060AB70 | 0x0060BA00 | `GameMap::GetItems()` | lua | alta |
| 0x0060AD10 | 0x0060BBA0 | `GameMap::GetRandomVoice()` | lua | alta |
| 0x0060AD70 | 0x0060BC00 | `GameMap::GetPawn(int)` | lua | alta |
| 0x0060ADA0 | 0x0060BC30 | `GameMap::GetAnotherVoice(int, int)` | lua | alta |
| 0x0060ADD0 | 0x0060BC60 | `GameMap::IsEvent(int)` | lua | alta |
| 0x0060AE10 | 0x0060BCA0 | `GameMap::TriggerSound(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0060AEE0 | 0x0060BD70 | `GameMap::GetEventCount(int)` | lua | alta |
| 0x0060CA40 | 0x0060D8C0 | `GameMap::GetPower()` | lua | alta |
| 0x0060CAE0 | 0x0060D960 | `GameMap::GetTeamTurn()` | lua | alta |
| 0x0060CB20 | 0x0060D9A0 | `GameMap::GetSector()` | lua | alta |
| 0x0060CB60 | 0x0060D9E0 | `GameMap::ModifyPowerGrid(int)` | lua | alta |
| 0x0060D3B0 | 0x0060E230 | `GameMap::AddPermaTip(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point)` | lua | alta |
| 0x0060D450 | 0x0060E2D0 | `GameMap::ClearTips()` | lua | alta |
| 0x0060D500 | 0x0060E380 | `GameMap::IsTip()` | lua | alta |
| 0x0060D540 | 0x0060E3C0 | `GameMap::AddInterfaceTip(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point, bool)` | lua | alta |
| 0x0060D650 | 0x0060E4D0 | `GameMap::AddVerticalInterfaceTip(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point, bool)` | lua | alta |
| 0x0060D770 | 0x0060E5F0 | `GameMap::AddTip(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point)` | lua | alta |
| 0x0060D910 | 0x0060E790 | `GameMap::AddTutorial(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point)` | lua | alta |
| 0x0060D9A0 | 0x0060E820 | `GameMap::AddVoicePopup(VoicePopup)` | lua | alta |
| 0x0060DA20 | 0x0060E8A0 | `GameMap::IsVoicePopup()` | lua | alta |
| 0x0060DA80 | 0x0060E900 | `GameMap::BlockNextTurn()` | lua | alta |
| 0x0060DAA0 | 0x0060E920 | `GameMap::AddNote(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0060E8B0 | 0x0060F730 | `GameMap::IsMechClass(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0060E930 | 0x0060F7B0 | `GameMap::GetCorp()` | lua | alta |
| 0x0060E950 | 0x0060F7D0 | `GameMap::GetSavedCorp()` | lua | alta |
| 0x0060E970 | 0x0060F7F0 | `GameMap::GetSquad()` | lua | alta |
| 0x00625130 | 0x00625FF0 | `SkillManager::SetPriorityTarget(Point)` | lua | alta |
| 0x00625150 | 0x00626010 | `SkillManager::GetShotsRemaining()` | lua | alta |
| 0x006255B0 | 0x00626470 | `SkillManager::ClearQueuedShot()` | lua~ | media |
| 0x00625D30 | 0x00626BF0 | `SkillManager::GetTarget()` | lua | alta |
| 0x00625EB0 | 0x00626D70 | `SkillManager::ResetUses()` | lua | alta |
| 0x00626480 | 0x00627340 | `SkillManager::GetFirstClick()` | lua | alta |
| 0x00628230 | 0x006290E0 | `Pawn::SetCustomAnim(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00629440 | 0x0062A2F0 | `Pawn::IsBusy()` | lua | alta |
| 0x00629530 | 0x0062A3E0 | `Pawn::IsDead()` | lua | alta |
| 0x00629540 | 0x0062A3F0 | `Pawn::IsCorpse()` | lua | alta |
| 0x006295B0 | 0x0062A460 | `Pawn::Damage(SpaceDamage, int)` | texto | media |
| 0x0062BF90 | 0x0062CE40 | `Pawn::ModifyHealth(int, bool, int)` | lua | alta |
| 0x0062C5F0 | 0x0062D4A0 | `Pawn::IsTurnShield()` | manual | media |
| 0x0062C650 | 0x0062D500 | `Pawn::SetHealth(int)` | lua | alta |
| 0x0062C910 | 0x0062D7C0 | `Pawn::SetSpace(Point, bool)` | manual | alta |
| 0x0062CA80 | 0x0062D930 | `Pawn::SetSpace(Point)` | manual | alta |
| 0x0062CAA0 | 0x0062D950 | `Pawn::IsPushBlocked(int)` | manual | alta |
| 0x0062CB10 | 0x0062D9C0 | `Pawn::IsPushBlocked()` | manual | media |
| 0x0062CC00 | 0x0062DAB0 | `Pawn::Push(int)` | texto | media |
| 0x0062CDD0 | 0x0062DC70 | `Pawn::FinishPush()` | texto | media |
| 0x0062F2D0 | 0x00630170 | `Pawn::SetTeam(int)` | lua | alta |
| 0x0062F2E0 | 0x00630180 | `Pawn::SetInvisible(bool)` | lua | alta |
| 0x0062F2F0 | 0x00630190 | `Pawn::IsForceField()` | manual | media |
| 0x0062F380 | 0x00630220 | `Pawn::GetTurnCount()` | lua | alta |
| 0x0062F700 | 0x006305A0 | `Pawn::GetPathingProfile()` | texto | media |
| 0x0062F840 | 0x006306E0 | `Pawn::IsPlayer()` | lua | alta |
| 0x0062F870 | 0x00630710 | `Pawn::IsEnemy()` | lua | alta |
| 0x006313E0 | 0x00632280 | `Pawn::IsBoosted()` | lua | alta |
| 0x00631490 | 0x00632330 | `Pawn::IsGrappled()` | lua | alta |
| 0x00631B20 | 0x006329C0 | `Pawn::GetPilotName(int)` | lua | alta |
| 0x00631E20 | 0x00632CC0 | `Pawn::GetPilotLanguage()` | lua | alta |
| 0x00632270 | 0x00633110 | `Pawn::Move(Point)` | lua | alta |
| 0x00632670 | 0x00633510 | `Pawn::SetManualPath(std::__ndk1::vector<Point, std::__ndk1::allocator<Point> >, bool)` | texto | media |
| 0x00632750 | 0x006335F0 | `Pawn::Leap(std::__ndk1::vector<Point, std::__ndk1::allocator<Point> >, bool, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | texto | media |
| 0x006329F0 | 0x00633890 | `Pawn::Charge(vector<Point>, bool, string)` | manual | media |
| 0x00632B40 | 0x006339E0 | `Pawn::Melee(SpaceDamage)` | manual | alta |
| 0x006335F0 | 0x00634490 | `Pawn::SetForceField(bool)` | texto | alta |
| 0x00633D60 | 0x00634C00 | `Pawn::SetActive(bool)` | lua | alta |
| 0x00633EA0 | 0x00634D40 | `Pawn::FireWeapon(Point, int)` | lua | alta |
| 0x00635F60 | 0x00636E00 | `Pawn::GetDangerScore()` | lua | alta |
| 0x00635FA0 | 0x00636E40 | `Pawn::GetSpace()` | lua | alta |
| 0x006361D0 | 0x00637070 | `Pawn::SpawnAnimation()` | lua | alta |
| 0x006362A0 | 0x00637140 | `Pawn::IsDeadly(std::vector<SpaceDamage>)` | manual | alta |
| 0x00636550 | 0x006373F0 | `Pawn::IsDeadly(SpaceDamage)` | lua | alta |
| 0x006365D0 | 0x00637470 | `Pawn::IsBurrower()` | lua | alta |
| 0x00636610 | 0x006374B0 | `Pawn::Burrow(Point, bool)` | manual | media |
| 0x006366E0 | 0x00637580 | `Pawn::SetPowered(bool)` | lua | alta |
| 0x00636970 | 0x00637810 | `Pawn::SetFrozen(bool)` | lua | alta |
| 0x00636B70 | 0x00637A10 | `Pawn::IsFrozen()` | lua | alta |
| 0x00636E10 | 0x00637CB0 | `Pawn::SetMissionCritical(bool)` | lua | alta |
| 0x00636E20 | 0x00637CC0 | `Pawn::Retreat()` | lua | alta |
| 0x006370D0 | 0x00637F70 | `Pawn::GetId()` | lua | alta |
| 0x00637210 | 0x006380B0 | `Pawn::SetInfected(bool)` | lua | alta |
| 0x006372F0 | 0x00638190 | `Pawn::IsInfected()` | lua | alta |
| 0x00637370 | 0x00638210 | `Pawn::IsPowered()` | lua | alta |
| 0x006373D0 | 0x00638270 | `Pawn::SetMutation(int)` | lua | alta |
| 0x00637610 | 0x006384B0 | `Pawn::IsAvoidingMines()` | lua | alta |
| 0x00637650 | 0x006384F0 | `Pawn::IsAbility(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x006378A0 | 0x00638740 | `Pawn::AddMoveBonus(int)` | lua | alta |
| 0x00638BD0 | 0x00639A70 | `Pawn::IsFireImmune()` | texto | alta |
| 0x00638D50 | 0x00639BF0 | `Pawn::IsDamaged()` | lua | alta |
| 0x00639040 | 0x00639EE0 | `Pawn::IsMech()` | lua | alta |
| 0x00639050 | 0x00639EF0 | `Pawn::SetMech()` | lua | alta |
| 0x00639060 | 0x00639F00 | `Pawn::SetOnFire(bool)` | texto | media |
| 0x006397C0 | 0x0063A660 | `Pawn::SetAcid(bool)` | lua | alta |
| 0x006398D0 | 0x0063A770 | `Pawn::IsAcid()` | lua | alta |
| 0x00639A50 | 0x0063A8F0 | `Pawn::Kill(bool)` | lua | alta |
| 0x00639AC0 | 0x0063A960 | `Pawn::IsTempUnit()` | lua | alta |
| 0x00639BF0 | 0x0063AA90 | `Pawn::Fall(int)` | lua | alta |
| 0x00639DA0 | 0x0063AC40 | `Pawn::FlyAway()` | lua | alta |
| 0x00639E10 | 0x0063ACB0 | `Pawn::IsPsionAffected(int)` | texto | alta |
| 0x00639FC0 | 0x0063AE60 | `Pawn::GetTeam()` | lua | alta |
| 0x0063A250 | 0x0063B0F0 | `Pawn::IsGuarding()` | lua | alta |
| 0x0063A260 | 0x0063B100 | `Pawn::SetBoosted(bool)` | lua | alta |
| 0x0063A450 | 0x0063B2F0 | `Pawn::IsUndoPossible()` | lua | alta |
| 0x0063A860 | 0x0063B700 | `Pawn::GetMoveSpeed()` | lua | alta |
| 0x0063A8B0 | 0x0063B750 | `Pawn::IsNonGridStructure()` | lua | alta |
| 0x0063A8F0 | 0x0063B790 | `Pawn::GetBaseMove()` | lua | alta |
| 0x0063AB10 | 0x0063B9B0 | `Pawn::SetMoveSpeed(int)` | lua | alta |
| 0x0063AB50 | 0x0063B9F0 | `Pawn::GetPersonality()` | lua | alta |
| 0x0063ABF0 | 0x0063BA90 | `Pawn::SetNeutral(bool)` | lua | alta |
| 0x0063AC00 | 0x0063BAA0 | `Pawn::IsFlying()` | lua | alta |
| 0x0063AF60 | 0x0063BE00 | `Pawn::IsTeleporter()` | lua | alta |
| 0x0063AFC0 | 0x0063BE60 | `Pawn::IsJumper()` | lua | alta |
| 0x0063B020 | 0x0063BEC0 | `Pawn::IsActive()` | lua | alta |
| 0x0063C0B0 | 0x0063CF50 | `Pawn::IsRanged()` | lua | alta |
| 0x0063C100 | 0x0063CFA0 | `Pawn::IsArmor()` | manual | media |
| 0x0063F3B0 | 0x00640250 | `Pawn::GetType()` | lua | alta |
| 0x00641560 | 0x006423F0 | `PawnFactory::CreatePawn(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | texto | media |
| 0x006528E0 | 0x00653760 | `SkillEffect::GetDamageCount()` | lua | alta |
| 0x00652900 | 0x00653780 | `SkillEffect::GetDamage(int)` | lua | alta |
| 0x00652920 | 0x006537A0 | `Squad::IsEmpty()` | lua | alta |
| 0x00652A50 | 0x006538D0 | `SkillEffect::AddDamage(SpaceDamage)` | lua | alta |
| 0x00652AD0 | 0x00653950 | `SkillEffect::AddVoice(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00652BC0 | 0x00653A40 | `SkillEffect::AddQueuedVoice(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00652CB0 | 0x00653B30 | `SkillEffect::AddQueuedDamage(SpaceDamage)` | lua | alta |
| 0x00652D30 | 0x00653BB0 | `SkillEffect::AddGrapple(Point, Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00652E30 | 0x00653CB0 | `SkillEffect::GetQueuedCount()` | lua | alta |
| 0x00652E50 | 0x00653CD0 | `SkillEffect::AddBurst(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00653100 | 0x00653F80 | `SkillEffect::GetQueuedDamage(int)` | lua | alta |
| 0x00653120 | 0x00653FA0 | `SkillEffect::AddDelay(float)` | lua | alta |
| 0x00653AE0 | 0x00654960 | `SkillEffect::AddMove(PointList, float)` | lua | alta |
| 0x00653BD0 | 0x00654A50 | `SkillEffect::AddLeap(PointList, float)` | lua | alta |
| 0x00653E00 | 0x00654C80 | `SkillEffect::AddBurrow(PointList, float)` | lua | alta |
| 0x00653E90 | 0x00654D10 | `SkillEffect::AddCharge(PointList, float)` | lua | alta |
| 0x00653F20 | 0x00654DA0 | `SkillEffect::AddAirstrike(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654030 | 0x00654EB0 | `SkillEffect::AddReverseAirstrike(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654140 | 0x00654FC0 | `SkillEffect::AddDropper(SpaceDamage, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654220 | 0x006550A0 | `SkillEffect::AddScript(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654500 | 0x00655380 | `SkillEffect::AddEmitter(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654580 | 0x00655400 | `SkillEffect::AddQueuedScript(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654670 | 0x006554F0 | `SkillEffect::AddSound(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654760 | 0x006555E0 | `SkillEffect::AddTeleport(Point, Point, float)` | lua | alta |
| 0x00654960 | 0x006557E0 | `SkillEffect::AddQueuedCharge(PointList, float)` | lua | alta |
| 0x006549F0 | 0x00655870 | `SkillEffect::AddQueuedMove(PointList, float)` | lua | alta |
| 0x00654AE0 | 0x00655960 | `SkillEffect::AddMelee(Point, SpaceDamage)` | lua | alta |
| 0x00654B50 | 0x006559D0 | `SkillEffect::AddMelee(Point, SpaceDamage, float)` | lua | alta |
| 0x00654C00 | 0x00655A80 | `SkillEffect::AddQueuedMelee(Point, SpaceDamage)` | lua | alta |
| 0x00654C70 | 0x00655AF0 | `SkillEffect::AddQueuedMelee(Point, SpaceDamage, float)` | lua | alta |
| 0x00654D20 | 0x00655BA0 | `SkillEffect::AddBoardShake(float)` | lua | alta |
| 0x00654EC0 | 0x00655D40 | `SkillEffect::AddBounce(Point, int)` | lua | alta |
| 0x00655F10 | 0x00656D90 | `SpaceDamage::IsMovement()` | lua | alta |
| 0x00655F40 | 0x00656DC0 | `SpaceDamage::MoveStart()` | lua | alta |
| 0x00655F80 | 0x00656E00 | `SpaceDamage::MoveEnd()` | lua | alta |
| 0x00655FA0 | 0x00656E20 | `SpaceDamage::AddVoice(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | texto | media |
| 0x00656140 | 0x00656FC0 | `SpaceDamage::GetProjectileArc(WeaponTarget*)` | vizinhos | media |
| 0x00656FD0 | 0x00657E50 | `Squad::AddPawn(Pawn*)` | lua | alta |
| 0x00675930 | 0x006767B0 | `TipData::IsValid()` | lua | alta |
| 0x00675C70 | 0x00676AF0 | `ActiveLua (slot 1: SetPawn/SetBoard/SetGame com nil)` | manual | alta |
| 0x00675D00 | 0x00676B80 | `ActiveLua::GetScripts()` | manual | alta |
| 0x00676020 | 0x00676EE0 | `ActiveLua::BindDefinitions()` | texto | alta |
| 0x006E3080 | 0x006E3F20 | `luabind::open(lua_State*)` | texto | alta |
