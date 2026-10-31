# Nomes das funções do PC vindos do Android (libmain.so 1.2.92)

Gerado por `re/tools/match_android_symbols.py` + `nomes_manual.csv` (`re/tools/gen_nomes_resumo.py`).
O `libmain.so` fica no PC do usuário, fora do repositório; aqui só há nomes e endereços.

* casamentos automáticos: 1192 ({'texto': 516, 'vtable': 373, 'lua': 259, 'vizinhos': 40, 'lua~': 4}); manuais: 38
* endereços de código citados no projeto: 540; com nome agora: 332

## Validação
Cada sinal foi conferido contra os outros: vftable × Lua 8/9; texto × Lua/vftable 62/65;
vizinhos (deixando cada casamento de fora) 68/69. Alinhar a ordem das chamadas falhou (0/72) e não é usado.
Confiança `media` = sinal mais fraco (1 string, nome Lua aproximado, vizinhos ou leitura manual).

## Achados
* A casa do tabuleiro é `BoardSpace`; a base de busca é `GridSearchable` (Dijkstra = `CalculateDijkstraMap`, A* = `GetAstarPath`).
* `Pawn::IsMassive()` existe; o byte +0x99D do PC (água não afoga, gelo danificado não mata) cumpre esse papel (inlinado no PC).
* Fila de efeitos: `Board::AddEffect` → `StackEffect` → `UpdateStackedEffects` → `ApplyEffect` (executor).

## Funções citadas no projeto

| PC | Nome no Android | Como | Confiança |
|---|---|---|---|
| 0x0044AC60 | `LuaEnv::BindUniversalDefinitions()` | texto | alta |
| 0x0044D1A0 | `LuaEnv::RefreshScripts()` | texto | media |
| 0x0046C4E0 | `GL_Color::WithAlpha(float) const` | lua | alta |
| 0x0046C510 | `GL_Color::Normalize() const` | lua | alta |
| 0x004BC350 | `Point::Manhattan(Point) const` | lua | alta |
| 0x004BC380 | `Point::GetString()` | lua | alta |
| 0x004BC510 | `Point::GetLuaString()` | lua | alta |
| 0x004CD680 | `GridSearchable::GetSingleCornerPath(Point, Point, int)` | manual | media |
| 0x004CDA00 | `GridSearchable::GetReachablePoints(Point, int, int)` | manual | alta |
| 0x004CDCD0 | `GridSearchable::GetDistance(Point, Point, int)` | lua | alta |
| 0x004CDE60 | `GridSearchable::CalculateDijkstraMap(Point, int, int, bool)` | manual | alta |
| 0x004CE630 | `GridSearchable::GetAstarPath(Point, Point, int)` | manual | alta |
| 0x004D48D0 | `BoardManager::IsCursor()` | vtable | alta |
| 0x004DFFC0 | `ValueBar::AnimatedModify(int)` | manual | media |
| 0x004E01F0 | `ValueBar::GetValue() const` | lua | alta |
| 0x004E0200 | `ValueBar::GetMax()` | lua | alta |
| 0x004E6260 | `CApp::OnRequestExit()` | vtable | alta |
| 0x004F6410 | `Pawn::GetHealth()` | lua | alta |
| 0x00503B60 | `Board::GetTerrain(Point)` | lua | alta |
| 0x0055D260 | `PylonAnimation::PylonAnimation(SpaceDamage, WeaponTarget*, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | texto | alta |
| 0x0055DB50 | `GameMap::AddEvent(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0055DB70 | `SkillManager::IsSelected()` | lua | alta |
| 0x0055DC10 | `GridSearchable::GetSize() const` | manual | alta |
| 0x0055F030 | `Board::IsBlocked(Point, int)` | lua | alta |
| 0x0055F0D0 | `Board::IsTerrain(Point, int)` | lua | alta |
| 0x0055F130 | `Board::GetHealth(Point)` | lua | alta |
| 0x0055F190 | `Board::SetHealth(Point, int, int)` | lua | alta |
| 0x0055F200 | `Board::DamageSpace(SpaceDamage)` | lua | alta |
| 0x0055F5B0 | `Board::DamageSpace(Point, int)` | lua | alta |
| 0x0055F680 | `Board::PushDamage(Point, int)` | manual | alta |
| 0x0055F7B0 | `Board::AddEffect(SkillEffect)` | lua | alta |
| 0x0055F970 | `Board::AddEffect(SpaceDamage)` | lua | alta |
| 0x005601C0 | `Board::ApplyEffect(SkillEffect, int)` | texto | alta |
| 0x00560EA0 | `Board::RemovePawn(Pawn*)` | lua | alta |
| 0x00561080 | `Board::RemovePawn(Point)` | lua | alta |
| 0x00561280 | `Board::GetLeaveCost(int, int, int)` | manual | alta |
| 0x005612A0 | `Board::GetEnterCost(int, int, int)` | manual | alta |
| 0x00561300 | `Board::IsEnterable(int, int, int)` | manual | alta |
| 0x00561370 | `Board::IsValid(Point)` | lua | alta |
| 0x00564D20 | `Board::MarkFlashing(Point, bool)` | lua | alta |
| 0x00564EC0 | `Board::ClearSpace(Point)` | lua | alta |
| 0x00565060 | `Board::SetTerrain(Point, int)` | lua | alta |
| 0x00565080 | `Board::SetCustomTile(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00565140 | `Board::GetCustomTile(Point)` | lua | alta |
| 0x005651E0 | `Board::SetSmoke(Point, bool, bool)` | lua | alta |
| 0x00565240 | `Board::AddAlert(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x005652D0 | `Board::SetAcid(Point, bool)` | lua | alta |
| 0x005653C0 | `Board::IsVines(Point)` | lua | alta |
| 0x005653E0 | `Board::IsSmoke(Point)` | lua | alta |
| 0x00565490 | `Board::GetPawns(int)` | lua | alta |
| 0x00565570 | `Board::GetBuildingsLua()` | lua~ | media |
| 0x00566170 | `Board::GetEnemyCount()` | lua | alta |
| 0x00567700 | `Board::GetPawn(int)` | lua | alta |
| 0x00567750 | `Board::GetPawn(Point, bool)` | lua | alta |
| 0x00567840 | `Board::IsPawnAlive(int)` | lua | alta |
| 0x00567970 | `Board::GetPawnSpace(int)` | lua | alta |
| 0x00567A10 | `Board::SetTerrainIcon(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00567AD0 | `Board::RandomizeTerrain(int, int)` | lua | alta |
| 0x00567C50 | `Board::IsFire(Point)` | lua | alta |
| 0x00567CB0 | `Board::IsAcid(Point)` | lua | alta |
| 0x00567D00 | `Board::IsBuilding(Point)` | lua | alta |
| 0x00567E30 | `Board::SpawnPawn(Pawn*, Point)` | lua | alta |
| 0x00568250 | `Board::SpawnPawn(Pawn*)` | lua | alta |
| 0x005686D0 | `Board::SpawnQueued()` | lua | alta |
| 0x00568860 | `Board::IsCrackable(Point)` | lua | alta |
| 0x00568950 | `Board::GetPawnTeam(Point)` | lua | alta |
| 0x005689C0 | `Board::GetPawnTeam(int, int)` | lua | alta |
| 0x005689E0 | `Board::IsBusy()` | lua | alta |
| 0x005689F0 | `Board::GetBusyState()` | lua | alta |
| 0x00568CF0 | `Board::UpdateStackedEffects()` | texto | alta |
| 0x00569920 | `Board::IsCracked(Point)` | lua | alta |
| 0x00569970 | `Board::SetCracked(Point, bool)` | lua | alta |
| 0x0056B240 | `Board::SetBlockSpawn(Point, int)` | lua~ | media |
| 0x0056B270 | `Board::ClearBlockSpawns()` | lua | alta |
| 0x0056B520 | `Board::AddTeleport(Point, Point)` | lua | alta |
| 0x0056BE90 | `Board::SetLava(Point, bool)` | lua | alta |
| 0x0056BED0 | `Board::GetDeployLocScore(Point)` | lua | alta |
| 0x0056BFC0 | `Board::GetTurn()` | lua | alta |
| 0x0056BFD0 | `Board::IsSafe(Point)` | lua | alta |
| 0x0056C710 | `Board::SetWeather(int, int, Point, Point, float)` | lua | alta |
| 0x0056C750 | `Board::StopWeather()` | lua | alta |
| 0x0056C780 | `Board::SetFrozen(Point, bool)` | lua | alta |
| 0x0056C7E0 | `Board::IsFrozen(Point)` | lua | alta |
| 0x0056CAF0 | `Board::GetZone(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0056D540 | `Board::Bounce(Point, int)` | lua | alta |
| 0x0056D5B0 | `Board::IsSpawning(Point)` | lua | alta |
| 0x0056D860 | `Board::IsTargeted(Point)` | lua | alta |
| 0x0056D930 | `Board::Ping(Point, GL_Color)` | lua | alta |
| 0x0056D9C0 | `Board::AddPawn(Pawn*, Point)` | lua | alta |
| 0x0056DE50 | `Board::AddPawn(Pawn*)` | lua | alta |
| 0x0056DFD0 | `Board::AddPawn(Pawn*, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | texto | media |
| 0x0056E590 | `Board::Fade(int)` | lua | alta |
| 0x0056EA40 | `Board::Slide(int)` | lua | alta |
| 0x0056ED50 | `Board::IsPawnTeam(Point, int)` | lua | alta |
| 0x0056EDA0 | `Board::IsPawnTeam(int, int, int)` | lua | alta |
| 0x0056EE50 | `Board::IsPawnSpace(Point)` | lua | alta |
| 0x0056EE70 | `Board::IsPawnSpace(Point, bool)` | lua | alta |
| 0x0056F170 | `Board::SetPopulated(bool, Point)` | lua | alta |
| 0x0056F940 | `Board::GetBuildingCount(int)` | lua | alta |
| 0x0056FB20 | `Board::IsEdge(Point)` | lua | alta |
| 0x00570430 | `Board::LockBomb()` | lua | alta |
| 0x00570660 | `Board::MarkSpaceColor(Point, GL_Color)` | lua | alta |
| 0x005706C0 | `Board::MarkSpaceSimpleColor(Point, GL_Color)` | lua | alta |
| 0x005706E0 | `Board::MarkSpaceDamage(SpaceDamage)` | lua | alta |
| 0x005707C0 | `Board::GetPawnCount(int)` | lua | alta |
| 0x00570990 | `Board::MarkSpaceImage(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, GL_Color)` | lua | alta |
| 0x00570E00 | `Board::GetPawn(Point)` | lua | alta |
| 0x00571680 | `Board::IsPod(Point)` | lua | alta |
| 0x005716F0 | `Board::IsItem(Point)` | lua | alta |
| 0x00571750 | `Board::IsDangerousItem(Point)` | lua | alta |
| 0x005717A0 | `Board::IsDangerous(Point)` | lua | alta |
| 0x00571870 | `Board::GetItem(Point)` | lua | alta |
| 0x00571930 | `Board::SetDangerous(Point)` | lua | alta |
| 0x00571950 | `Board::SetItem(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00571B20 | `Board::IsPushBlocked(Point, int)` | manual | alta |
| 0x005723D0 | `Board::IsEnvironmentDanger(Point)` | lua | alta |
| 0x00572680 | `Board::GetRandomBuilding(bool)` | lua | alta |
| 0x005728D0 | `Board::GetInfectedCount()` | lua | alta |
| 0x00572A50 | `Board::GetAnotherPlayerPawn(int, int)` | lua | alta |
| 0x00572B50 | `Board::GetMechDamage()` | lua | alta |
| 0x00572D90 | `Board::GetDistanceToPawn(Point, int)` | lua | alta |
| 0x00573030 | `Board::GetDistanceToBuilding(Point)` | lua | alta |
| 0x005730B0 | `Board::AddAnimation(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00573160 | `Board::GetSimpleReachable(Point, int, bool)` | lua | alta |
| 0x005731F0 | `Board::GetSimplePath(Point, Point)` | lua | alta |
| 0x00573280 | `Board::GetReachablePointList(Point, int, int)` | lua~ | media |
| 0x005733D0 | `Board::GetPath(Point, Point, int)` | lua | alta |
| 0x00573490 | `Board::AddBurst(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00573540 | `Board::IsWall(Point, int, int)` | lua | alta |
| 0x005735E0 | `Board::SetWall(bool, Point, int)` | lua | alta |
| 0x00573680 | `Board::AddUniqueBuilding(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00573820 | `Board::IsUniqueBuilding(Point)` | lua | alta |
| 0x00573950 | `Board::IsDamaged(Point)` | lua | alta |
| 0x005739E0 | `Board::IsSkillDeadly(SpaceDamage, Pawn*)` | manual | alta |
| 0x00573B80 | `Board::StartMechTravel()` | lua | alta |
| 0x00573EB0 | `Board::StartPopEvent(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0057EAC0 | `SkillManager::GetArmedWeaponId()` | lua | alta |
| 0x0057EAD0 | `SkillManager::IsWeaponArmed()` | lua | alta |
| 0x0059C5C0 | `BoardSpace::CheckTerrainDangers()` | texto | alta |
| 0x0059E8A0 | `BoardSpace::SetFrozen(bool)` | vizinhos | media |
| 0x0059EBE0 | `BoardSpace::SetForceField()` | texto | alta |
| 0x0059EE20 | `BoardSpace::RemoveForceField()` | manual | alta |
| 0x0059F050 | `BoardSpace::IsPawnSpace(bool)` | manual | alta |
| 0x0059F260 | `BoardSpace::IsWalkable(int)` | manual | media |
| 0x0059F3A0 | `BoardSpace::GetEnterPenalty(int)` | manual | media |
| 0x0059F7C0 | `BoardSpace::IsWall(int)` | manual | alta |
| 0x005A03E0 | `BoardSpace::ClearItem()` | texto | alta |
| 0x005A0CB0 | `BoardSpace::SetTerrain(int)` | texto | alta |
| 0x005A10B0 | `BoardSpace::AddBuilding()` | manual | alta |
| 0x005AAA10 | `BoardSpace::IsPushBlocked(int)` | manual | alta |
| 0x005AABD0 | `BoardSpace::SetAcid(bool)` | manual | alta |
| 0x005AAC40 | `BoardSpace::SetCracked(bool)` | texto | media |
| 0x005AAE10 | `BoardSpace::DamageTerrain(int)` | texto | media |
| 0x005AB580 | `BoardSpace::SetFire(bool)` | texto | alta |
| 0x005AB740 | `BoardSpace::SetSmoke(bool, bool)` | texto | media |
| 0x005ABA90 | `BoardSpace::DamageSpace(SpaceDamage, int)` | texto | alta |
| 0x005ADBB0 | `BoardSpace::PushSpace(int)` | manual | alta |
| 0x005AE490 | `BoardSpace::GetPawn(bool)` | manual | alta |
| 0x005AE510 | `BoardSpace::IsTerrainBlocking(int)` | manual | alta |
| 0x005AE5D0 | `BoardSpace::IsTerrain(int)` | manual | alta |
| 0x005AE6B0 | `BoardSpace::IsCrackable()` | manual | alta |
| 0x005B99F0 | `RepObj::Failed()` | lua | alta |
| 0x005B9B20 | `RepObj::GetLuaString()` | lua | alta |
| 0x005C4B90 | `PowerBar::Damage(int)` | texto | media |
| 0x005CEA20 | `ChoiceEvent::SetEffect(Effect)` | lua | alta |
| 0x005CEA80 | `ChoiceEvent::SetStartText(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x005CFB30 | `ChoiceEvent::AddChoice(Choice)` | lua | alta |
| 0x005CFCA0 | `ChoiceEvent::GetText()` | lua | alta |
| 0x005CFCC0 | `ChoiceEvent::SetHiddenEffects(bool)` | lua | alta |
| 0x005CFCD0 | `ChoiceEvent::SetUniqueMech(bool)` | lua | alta |
| 0x005F04B0 | `Effect::AddReward(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x006088E0 | `GameMap::AddWeapon(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x006089B0 | `GameMap::RemoveItem(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00608A80 | `GameMap::AddPilot(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00609280 | `GameMap::IsIslandUnlocked(int)` | lua | alta |
| 0x0060AB70 | `GameMap::GetItems()` | lua | alta |
| 0x0060AD10 | `GameMap::GetRandomVoice()` | lua | alta |
| 0x0060AD70 | `GameMap::GetPawn(int)` | lua | alta |
| 0x0060ADA0 | `GameMap::GetAnotherVoice(int, int)` | lua | alta |
| 0x0060ADD0 | `GameMap::IsEvent(int)` | lua | alta |
| 0x0060AE10 | `GameMap::TriggerSound(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0060AEE0 | `GameMap::GetEventCount(int)` | lua | alta |
| 0x0060CA40 | `GameMap::GetPower()` | lua | alta |
| 0x0060CAE0 | `GameMap::GetTeamTurn()` | lua | alta |
| 0x0060CB20 | `GameMap::GetSector()` | lua | alta |
| 0x0060CB60 | `GameMap::ModifyPowerGrid(int)` | lua | alta |
| 0x0060D3B0 | `GameMap::AddPermaTip(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point)` | lua | alta |
| 0x0060D450 | `GameMap::ClearTips()` | lua | alta |
| 0x0060D500 | `GameMap::IsTip()` | lua | alta |
| 0x0060D540 | `GameMap::AddInterfaceTip(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point, bool)` | lua | alta |
| 0x0060D650 | `GameMap::AddVerticalInterfaceTip(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point, bool)` | lua | alta |
| 0x0060D770 | `GameMap::AddTip(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point)` | lua | alta |
| 0x0060D910 | `GameMap::AddTutorial(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, Point)` | lua | alta |
| 0x0060D9A0 | `GameMap::AddVoicePopup(VoicePopup)` | lua | alta |
| 0x0060DA20 | `GameMap::IsVoicePopup()` | lua | alta |
| 0x0060DA80 | `GameMap::BlockNextTurn()` | lua | alta |
| 0x0060DAA0 | `GameMap::AddNote(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0060E8B0 | `GameMap::IsMechClass(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x0060E930 | `GameMap::GetCorp()` | lua | alta |
| 0x0060E950 | `GameMap::GetSavedCorp()` | lua | alta |
| 0x0060E970 | `GameMap::GetSquad()` | lua | alta |
| 0x00625130 | `SkillManager::SetPriorityTarget(Point)` | lua | alta |
| 0x00625150 | `SkillManager::GetShotsRemaining()` | lua | alta |
| 0x006255B0 | `SkillManager::ClearQueuedShot()` | lua~ | media |
| 0x00625D30 | `SkillManager::GetTarget()` | lua | alta |
| 0x00625EB0 | `SkillManager::ResetUses()` | lua | alta |
| 0x00626480 | `SkillManager::GetFirstClick()` | lua | alta |
| 0x00628230 | `Pawn::SetCustomAnim(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00629440 | `Pawn::IsBusy()` | lua | alta |
| 0x00629530 | `Pawn::IsDead()` | lua | alta |
| 0x00629540 | `Pawn::IsCorpse()` | lua | alta |
| 0x006295B0 | `Pawn::Damage(SpaceDamage, int)` | texto | media |
| 0x0062BF90 | `Pawn::ModifyHealth(int, bool, int)` | lua | alta |
| 0x0062C5F0 | `Pawn::IsTurnShield()` | manual | media |
| 0x0062C650 | `Pawn::SetHealth(int)` | lua | alta |
| 0x0062C910 | `Pawn::SetSpace(Point, bool)` | manual | alta |
| 0x0062CA80 | `Pawn::SetSpace(Point)` | manual | alta |
| 0x0062CAA0 | `Pawn::IsPushBlocked(int)` | manual | alta |
| 0x0062CB10 | `Pawn::IsPushBlocked()` | manual | media |
| 0x0062CC00 | `Pawn::Push(int)` | texto | media |
| 0x0062CDD0 | `Pawn::FinishPush()` | texto | media |
| 0x0062F2D0 | `Pawn::SetTeam(int)` | lua | alta |
| 0x0062F2E0 | `Pawn::SetInvisible(bool)` | lua | alta |
| 0x0062F2F0 | `Pawn::IsForceField()` | manual | media |
| 0x0062F380 | `Pawn::GetTurnCount()` | lua | alta |
| 0x0062F700 | `Pawn::GetPathingProfile()` | texto | media |
| 0x0062F840 | `Pawn::IsPlayer()` | lua | alta |
| 0x0062F870 | `Pawn::IsEnemy()` | lua | alta |
| 0x006313E0 | `Pawn::IsBoosted()` | lua | alta |
| 0x00631490 | `Pawn::IsGrappled()` | lua | alta |
| 0x00631B20 | `Pawn::GetPilotName(int)` | lua | alta |
| 0x00631E20 | `Pawn::GetPilotLanguage()` | lua | alta |
| 0x00632270 | `Pawn::Move(Point)` | lua | alta |
| 0x00632670 | `Pawn::SetManualPath(std::__ndk1::vector<Point, std::__ndk1::allocator<Point> >, bool)` | texto | media |
| 0x00632750 | `Pawn::Leap(std::__ndk1::vector<Point, std::__ndk1::allocator<Point> >, bool, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | texto | media |
| 0x006329F0 | `Pawn::Charge(vector<Point>, bool, string)` | manual | media |
| 0x00632B40 | `Pawn::Melee(SpaceDamage)` | manual | alta |
| 0x006335F0 | `Pawn::SetForceField(bool)` | texto | alta |
| 0x00633D60 | `Pawn::SetActive(bool)` | lua | alta |
| 0x00633EA0 | `Pawn::FireWeapon(Point, int)` | lua | alta |
| 0x00635F60 | `Pawn::GetDangerScore()` | lua | alta |
| 0x00635FA0 | `Pawn::GetSpace()` | lua | alta |
| 0x006361D0 | `Pawn::SpawnAnimation()` | lua | alta |
| 0x006362A0 | `Pawn::IsDeadly(std::vector<SpaceDamage>)` | manual | alta |
| 0x00636550 | `Pawn::IsDeadly(SpaceDamage)` | lua | alta |
| 0x006365D0 | `Pawn::IsBurrower()` | lua | alta |
| 0x00636610 | `Pawn::Burrow(Point, bool)` | manual | media |
| 0x006366E0 | `Pawn::SetPowered(bool)` | lua | alta |
| 0x00636970 | `Pawn::SetFrozen(bool)` | lua | alta |
| 0x00636B70 | `Pawn::IsFrozen()` | lua | alta |
| 0x00636E10 | `Pawn::SetMissionCritical(bool)` | lua | alta |
| 0x00636E20 | `Pawn::Retreat()` | lua | alta |
| 0x006370D0 | `Pawn::GetId()` | lua | alta |
| 0x00637210 | `Pawn::SetInfected(bool)` | lua | alta |
| 0x006372F0 | `Pawn::IsInfected()` | lua | alta |
| 0x00637370 | `Pawn::IsPowered()` | lua | alta |
| 0x006373D0 | `Pawn::SetMutation(int)` | lua | alta |
| 0x00637610 | `Pawn::IsAvoidingMines()` | lua | alta |
| 0x00637650 | `Pawn::IsAbility(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x006378A0 | `Pawn::AddMoveBonus(int)` | lua | alta |
| 0x00638BD0 | `Pawn::IsFireImmune()` | texto | alta |
| 0x00638D50 | `Pawn::IsDamaged()` | lua | alta |
| 0x00639040 | `Pawn::IsMech()` | lua | alta |
| 0x00639050 | `Pawn::SetMech()` | lua | alta |
| 0x00639060 | `Pawn::SetOnFire(bool)` | texto | media |
| 0x006397C0 | `Pawn::SetAcid(bool)` | lua | alta |
| 0x006398D0 | `Pawn::IsAcid()` | lua | alta |
| 0x00639A50 | `Pawn::Kill(bool)` | lua | alta |
| 0x00639AC0 | `Pawn::IsTempUnit()` | lua | alta |
| 0x00639BF0 | `Pawn::Fall(int)` | lua | alta |
| 0x00639DA0 | `Pawn::FlyAway()` | lua | alta |
| 0x00639E10 | `Pawn::IsPsionAffected(int)` | texto | alta |
| 0x00639FC0 | `Pawn::GetTeam()` | lua | alta |
| 0x0063A250 | `Pawn::IsGuarding()` | lua | alta |
| 0x0063A260 | `Pawn::SetBoosted(bool)` | lua | alta |
| 0x0063A450 | `Pawn::IsUndoPossible()` | lua | alta |
| 0x0063A860 | `Pawn::GetMoveSpeed()` | lua | alta |
| 0x0063A8B0 | `Pawn::IsNonGridStructure()` | lua | alta |
| 0x0063A8F0 | `Pawn::GetBaseMove()` | lua | alta |
| 0x0063AB10 | `Pawn::SetMoveSpeed(int)` | lua | alta |
| 0x0063AB50 | `Pawn::GetPersonality()` | lua | alta |
| 0x0063ABF0 | `Pawn::SetNeutral(bool)` | lua | alta |
| 0x0063AC00 | `Pawn::IsFlying()` | lua | alta |
| 0x0063AF60 | `Pawn::IsTeleporter()` | lua | alta |
| 0x0063AFC0 | `Pawn::IsJumper()` | lua | alta |
| 0x0063B020 | `Pawn::IsActive()` | lua | alta |
| 0x0063C0B0 | `Pawn::IsRanged()` | lua | alta |
| 0x0063C100 | `Pawn::IsArmor()` | manual | media |
| 0x0063F3B0 | `Pawn::GetType()` | lua | alta |
| 0x00641560 | `PawnFactory::CreatePawn(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | texto | media |
| 0x006528E0 | `SkillEffect::GetDamageCount()` | lua | alta |
| 0x00652900 | `SkillEffect::GetDamage(int)` | lua | alta |
| 0x00652920 | `Squad::IsEmpty()` | lua | alta |
| 0x00652A50 | `SkillEffect::AddDamage(SpaceDamage)` | lua | alta |
| 0x00652AD0 | `SkillEffect::AddVoice(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00652BC0 | `SkillEffect::AddQueuedVoice(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00652CB0 | `SkillEffect::AddQueuedDamage(SpaceDamage)` | lua | alta |
| 0x00652D30 | `SkillEffect::AddGrapple(Point, Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00652E30 | `SkillEffect::GetQueuedCount()` | lua | alta |
| 0x00652E50 | `SkillEffect::AddBurst(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | lua | alta |
| 0x00653100 | `SkillEffect::GetQueuedDamage(int)` | lua | alta |
| 0x00653120 | `SkillEffect::AddDelay(float)` | lua | alta |
| 0x00653AE0 | `SkillEffect::AddMove(PointList, float)` | lua | alta |
| 0x00653BD0 | `SkillEffect::AddLeap(PointList, float)` | lua | alta |
| 0x00653E00 | `SkillEffect::AddBurrow(PointList, float)` | lua | alta |
| 0x00653E90 | `SkillEffect::AddCharge(PointList, float)` | lua | alta |
| 0x00653F20 | `SkillEffect::AddAirstrike(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654030 | `SkillEffect::AddReverseAirstrike(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654140 | `SkillEffect::AddDropper(SpaceDamage, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654220 | `SkillEffect::AddScript(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654500 | `SkillEffect::AddEmitter(Point, std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654580 | `SkillEffect::AddQueuedScript(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654670 | `SkillEffect::AddSound(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >)` | lua | alta |
| 0x00654760 | `SkillEffect::AddTeleport(Point, Point, float)` | lua | alta |
| 0x00654960 | `SkillEffect::AddQueuedCharge(PointList, float)` | lua | alta |
| 0x006549F0 | `SkillEffect::AddQueuedMove(PointList, float)` | lua | alta |
| 0x00654AE0 | `SkillEffect::AddMelee(Point, SpaceDamage)` | lua | alta |
| 0x00654B50 | `SkillEffect::AddMelee(Point, SpaceDamage, float)` | lua | alta |
| 0x00654C00 | `SkillEffect::AddQueuedMelee(Point, SpaceDamage)` | lua | alta |
| 0x00654C70 | `SkillEffect::AddQueuedMelee(Point, SpaceDamage, float)` | lua | alta |
| 0x00654D20 | `SkillEffect::AddBoardShake(float)` | lua | alta |
| 0x00654EC0 | `SkillEffect::AddBounce(Point, int)` | lua | alta |
| 0x00655F10 | `SpaceDamage::IsMovement()` | lua | alta |
| 0x00655F40 | `SpaceDamage::MoveStart()` | lua | alta |
| 0x00655F80 | `SpaceDamage::MoveEnd()` | lua | alta |
| 0x00655FA0 | `SpaceDamage::AddVoice(std::__ndk1::basic_string<char, std::__ndk1::char_traits<char>, std::__ndk1::allocator<char> >, int)` | texto | media |
| 0x00656140 | `SpaceDamage::GetProjectileArc(WeaponTarget*)` | vizinhos | media |
| 0x00656FD0 | `Squad::AddPawn(Pawn*)` | lua | alta |
| 0x00675930 | `TipData::IsValid()` | lua | alta |
| 0x00675C70 | `ActiveLua::GetScripts()` | texto | alta |
| 0x00676020 | `ActiveLua::BindDefinitions()` | texto | alta |
| 0x006E3080 | `luabind::open(lua_State*)` | texto | alta |
