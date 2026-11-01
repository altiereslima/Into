-- Coleta do teste diferencial. O MESMO arquivo roda no jogo original (numa cópia da instalação, ver
-- preparar.py) e no motor Rust (itb-diff). Cada linha é "seção|chave|valor"; o comparador casa pela chave.
--
-- Variáveis definidas por quem carrega este arquivo:
--   DIF_SAIDA  caminho do arquivo de saída
--   DIF_MODO   "boot": coleta o que não depende de missão e encerra o processo
--              "missao": além disso, grava a preparação da missão, espera ela começar, coleta o tabuleiro e encerra
--              "rust": coleta o que não depende de missão; com DIF_MAPA, deixa DIF_COLETA_MISSAO para depois
--              que o Rust preparar a missão
--
-- Só chama o que o motor Rust já reconstruiu e o que não depende de haver missão: no original, um método
-- que use o tabuleiro sem haver tabuleiro derruba o jogo. A saída não tem buffer, então a última linha
-- gravada aponta a chamada culpada em caso de queda.

local saida = assert(io.open(DIF_SAIDA, "w"))
saida:setvbuf("no")

local function emit(secao, chave, valor)
	saida:write(secao, "|", tostring(chave), "|", tostring(valor), "\n")
end

-- Só o fato de ter falhado é comparado: as mensagens de erro de luabind e mlua são diferentes.
local function tenta(f, ...)
	local r = { pcall(f, ...) }
	if not r[1] then return "ERRO" end
	return unpack(r, 2)
end

local function ponto(p)
	if p == nil then return "nil" end
	return p.x .. "," .. p.y
end

local function sorted_keys(t)
	local ks = {}
	for k in pairs(t) do
		if type(k) == "string" then ks[#ks + 1] = k end
	end
	table.sort(ks)
	return ks
end

---------------------------------------------------------------------- globais simples
local function coleta_globais()
	for _, k in ipairs(sorted_keys(_G)) do
		local v = rawget(_G, k)
		local t = type(v)
		if t == "number" or t == "boolean" or t == "string" then
			emit("global", k, v)
		elseif t == "userdata" then
			local x = tenta(function() return v.x end)
			local r = tenta(function() return v.r end)
			if type(x) == "number" then
				emit("global", k, "Point " .. ponto(v))
			elseif type(r) == "number" then
				emit("global", k, "Color " .. v.r .. "," .. v.g .. "," .. v.b .. "," .. v.a)
			end
		end
	end
end

---------------------------------------------------------------------- tipos de valor
local function coleta_valores()
	local casos = {
		{ "vazio", function() return Point() end },
		{ "2,3", function() return Point(2, 3) end },
		{ "2.5,3.5", function() return Point(2.5, 3.5) end },
		{ "-1.5,0.5", function() return Point(-1.5, 0.5) end },
		{ "soma", function() return Point(1, 2) + Point(3, 4) end },
		{ "subtrai", function() return Point(1, 2) - Point(3, 4) end },
		{ "vezes", function() return Point(2, -3) * 3 end },
		{ "vezes_frac", function() return Point(3, 5) * 0.5 end },
	}
	for _, c in ipairs(casos) do
		local p = tenta(c[2])
		emit("point", c[1], p == "ERRO" and p or ponto(p))
	end
	emit("point", "manhattan", Point(1, 2):Manhattan(Point(-3, 7)))
	emit("point", "string", Point(5, -6):GetString())
	emit("point", "luastring", Point(5, -6):GetLuaString())
	emit("point", "igual", Point(1, 2) == Point(1, 2))
	emit("point", "diferente", Point(1, 2) == Point(2, 1))
	for x = -3, 3 do
		for y = -3, 3 do
			emit("direcao", x .. "," .. y, GetDirection(Point(x, y)))
		end
	end

	local rects = {
		{ "vazio", function() return Rect2D() end },
		{ "4", function() return Rect2D(1, 2, 3, 4) end },
		{ "frac", function() return Rect2D(1.5, 2.5, 3.5, 4.5) end },
		{ "2", function() return Rect2D(Point(1, 2), Point(3, 4)) end },
	}
	for _, c in ipairs(rects) do
		local r = tenta(c[2])
		emit("rect", c[1], r == "ERRO" and r or (r.x .. "," .. r.y .. "," .. r.w .. "," .. r.h))
	end

	local cores = {
		{ "vazio", function() return GL_Color() end },
		{ "3", function() return GL_Color(10, 20, 30) end },
		{ "4", function() return GL_Color(10, 20, 30, 0.5) end },
		{ "normaliza", function() return GL_Color(51, 102, 255):Normalize() end },
		{ "alfa", function() return GL_Color(10, 20, 30):WithAlpha(0.25) end },
	}
	for _, c in ipairs(cores) do
		local r = tenta(c[2])
		emit("cor", c[1], r == "ERRO" and r or (r.r .. "," .. r.g .. "," .. r.b .. "," .. r.a))
	end

	for _, n in ipairs({ "GetDifficulty", "IsRelease", "IsPhoneUI", "IsTouch", "IsNewMissions", "IsNewEquipment", "IsNewEnemies" }) do
		emit("plataforma", n, tostring(tenta(_G[n])))
	end
end

---------------------------------------------------------------------- rand
-- random_int(32768) devolve rand() inteiro. O comparador recupera o estado e prova que é a mesma sequência.
local function coleta_rand()
	for i = 1, 16 do
		emit("rand", string.format("%02d", i), random_int(32768))
	end
end

---------------------------------------------------------------------- SpaceDamage e SkillEffect
local CAMPOS = { "iDamage", "iPush", "iAcid", "iFire", "iSmoke", "iFrozen", "iShield", "iInjure", "iCrack", "iTerrain",
	"iPawnTeam", "fDelay", "bHide", "bHideIcon", "bHidePath", "bEvacuate", "bKO_Effect", "bSimpleMark",
	"sAnimation", "sImageMark", "sItem", "sPawn", "sScript", "sSound" }

local function dano(prefixo, d)
	emit("dano", prefixo .. ".loc", ponto(d.loc))
	for _, c in ipairs(CAMPOS) do
		emit("dano", prefixo .. "." .. c, tostring(d[c]))
	end
	emit("dano", prefixo .. ".IsMovement", tostring(d:IsMovement()))
	-- sem movimento, MoveStart e MoveEnd leem memória inválida no original
	if d:IsMovement() then
		emit("dano", prefixo .. ".MoveStart", ponto(d:MoveStart()))
		emit("dano", prefixo .. ".MoveEnd", ponto(d:MoveEnd()))
	end
end

local function coleta_dano()
	dano("vazio", SpaceDamage())
	dano("int", SpaceDamage(2))
	dano("point", SpaceDamage(Point(1, 2)))
	dano("point_int", SpaceDamage(Point(1, 2), 3))
	dano("point_int_int", SpaceDamage(Point(1, 2), 3, 1))

	local se = SkillEffect()
	emit("efeito", "origem", ponto(se.piOrigin))
	emit("efeito", "dono", se.iOwner)
	emit("efeito", "som", se.impact_sound)
	se:AddDamage(SpaceDamage(Point(1, 1), 2))
	se:AddQueuedDamage(SpaceDamage(Point(2, 2), 3))
	se:AddDelay(0.5)
	se:AddScript("x = 1")
	se:AddSound("/som")
	emit("efeito", "effect_n", se.effect:size())
	emit("efeito", "q_effect_n", se.q_effect:size())
	emit("efeito", "GetDamageCount", se:GetDamageCount())
	emit("efeito", "GetQueuedCount", se:GetQueuedCount())
	for i = 1, se.effect:size() do dano("effect" .. i, se.effect:index(i)) end
	for i = 1, se.q_effect:size() do dano("q_effect" .. i, se.q_effect:index(i)) end

	local objetivos = {
		{ "s_i", function() return Objective("t", 1) end },
		{ "s_i_i", function() return Objective("t", 1, 2) end },
	}
	for _, c in ipairs(objetivos) do
		local o = tenta(c[2])
		if o == "ERRO" then
			emit("objetivo", c[1], o)
		else
			emit("objetivo", c[1], tostring(o.text) .. "," .. tostring(o.rep) .. "," .. tostring(o.potential) .. "," .. tostring(o.category))
			local f = o:Failed()
			emit("objetivo", c[1] .. ".Failed", tostring(f.text) .. "," .. tostring(f.rep) .. "," .. tostring(f.potential) .. "," .. tostring(f.category))
		end
	end
end

---------------------------------------------------------------------- unidades
-- Consultas sobre a unidade fora de tabuleiro.
local CONSULTAS = { "GetHealth", "GetTeam", "GetType", "IsDead", "IsDamaged", "IsEnemy", "IsPlayer", "IsMech",
	"IsAcid", "IsFire", "IsFrozen", "IsInfected", "IsPowered", "IsGuarding", "IsGrappled",
	"IsFlying", "IsTeleporter", "IsJumper", "IsBurrower", "GetPathProf", "GetMoveSpeed", "IsShield", "IsBoosted", "IsCorpse",
	"IsNonGridStructure", "IsRanged", "IsTempUnit" }

-- Conta quantos rand() uma chamada gasta: recupera o estado do LCG a partir de valores de random_int(32768)
-- e procura o valor seguinte adiante na sequência. O produto é feito em partes porque a Lua usa double.
local function lcg(s)
	local baixo = s % 65536
	local alto = (s - baixo) / 65536
	local r = baixo * 214013 + (alto * 214013 % 65536) * 65536 + 2531011
	return r % 4294967296
end

local function saida_lcg(s) return math.floor(s / 65536) % 32768 end

local function recupera_estado()
	local a, b, c = random_int(32768), random_int(32768), random_int(32768)
	for alto = 0, 1 do
		for baixo = 0, 65535 do
			local s = alto * 2147483648 + a * 65536 + baixo
			local s1 = lcg(s)
			if saida_lcg(s1) == b then
				local s2 = lcg(s1)
				if saida_lcg(s2) == c then return s2 end
			end
		end
	end
end

-- Quantos rand() houve desde o estado `s`, dados os próximos dois valores. Devolve a contagem e o novo estado.
local function conta_rand(s)
	local v1, v2 = random_int(32768), random_int(32768)
	local t = s
	for n = 0, 400 do
		t = lcg(t)
		local t2 = lcg(t)
		if saida_lcg(t) == v1 and saida_lcg(t2) == v2 then return n, t2 end
	end
	return -1, nil
end

local function coleta_unidades()
	local nomes = {}
	for _, n in ipairs(PawnList) do nomes[#nomes + 1] = n end
	table.sort(nomes)
	emit("unidade", "total", #nomes)
	local estado = recupera_estado()
	for _, n in ipairs(nomes) do
		local p = PAWN_FACTORY:CreatePawn(n)
		local gastos
		gastos, estado = conta_rand(estado)
		emit("consumo", n, gastos)
		if estado == nil then estado = recupera_estado() end
		for _, c in ipairs(CONSULTAS) do
			-- grava a chave antes da chamada: em caso de queda, o arquivo mostra o que a causou
			saida:write("# ", n, ":", c, "\n")
			emit("unidade", n .. "." .. c, tostring(p[c](p)))
		end
		emit("unidade", n .. ".GetCustomPositionScore", tostring(p:GetCustomPositionScore(Point(1, 1))))
	end
end

---------------------------------------------------------------------- tabuleiro (precisa de missão)
local function lista_pontos(l)
	local t = {}
	for i = 1, l:size() do t[i] = ponto(l:index(i)) end
	return table.concat(t, " ")
end

local PERFIS = { 0, 1, 2, 3, 4, 7, 9 }

local function coleta_casas(secao)
	local tam = Board:GetSize()
	emit(secao, "tamanho", ponto(tam))
	for x = 0, tam.x - 1 do
		for y = 0, tam.y - 1 do
			local p = Point(x, y)
			local k = x .. "," .. y
			emit(secao, k .. ".terreno", Board:GetTerrain(p))
			for _, n in ipairs({ "IsBuilding", "IsFire", "IsSmoke", "IsVines", "IsAcid", "IsCracked", "IsEdge", "IsPod", "IsItem",
				"IsDangerous", "IsSpawning", "IsCrackable", "IsValid", "IsPawnSpace", "IsPowered", "IsFrozen", "IsDangerousItem",
				"IsTargeted", "IsDamaged", "IsSafe" }) do
				emit(secao, k .. "." .. n, tostring(Board[n](Board, p)))
			end
			emit(secao, k .. ".GetPawnTeam", Board:GetPawnTeam(p))
			emit(secao, k .. ".vida", Board:GetHealth(p))
			emit(secao, k .. ".dist_predio", Board:GetDistanceToBuilding(p))
			emit(secao, k .. ".dist_jogador", Board:GetDistanceToPawn(p, TEAM_PLAYER))
			emit(secao, k .. ".dist_inimigo", Board:GetDistanceToPawn(p, TEAM_ENEMY))
			emit(secao, k .. ".lava", tostring(Board:IsTerrain(p, TERRAIN_LAVA)))
			for _, perfil in ipairs(PERFIS) do
				emit(secao, k .. ".bloqueio" .. perfil, tostring(Board:IsBlocked(p, perfil)))
			end
		end
	end
end

local function coleta_caminhos()
	local origens = { Point(0, 0), Point(3, 5), Point(6, 2), Point(7, 7), Point(1, 6) }
	for _, o in ipairs(origens) do
		local k = ponto(o)
		for _, perfil in ipairs(PERFIS) do
			for _, alcance in ipairs({ 1, 3, 6 }) do
				emit("alcance", k .. ".p" .. perfil .. ".a" .. alcance, lista_pontos(Board:GetReachable(o, alcance, perfil)))
			end
			for _, d in ipairs(origens) do
				emit("caminho", k .. ">" .. ponto(d) .. ".p" .. perfil, lista_pontos(Board:GetPath(o, d, perfil)))
			end
		end
		for _, alcance in ipairs({ 1, 3 }) do
			emit("alcance", k .. ".simples" .. alcance, lista_pontos(Board:GetSimpleReachable(o, alcance, false)))
			emit("alcance", k .. ".simples_cantos" .. alcance, lista_pontos(Board:GetSimpleReachable(o, alcance, true)))
		end
		for _, d in ipairs(origens) do
			emit("caminho", k .. ">" .. ponto(d) .. ".simples", lista_pontos(Board:GetSimplePath(o, d)))
		end
	end
	for _, z in ipairs({ "enemy", "deployment", "nao_existe" }) do
		emit("zona", z, lista_pontos(Board:GetZone(z)))
	end
end

-- Efeito de todas as armas, sem aplicar. Grava o nome antes de chamar a arma: em caso de queda, o arquivo aponta qual foi.
local function coleta_armas(mech, alvo)
	local nomes = {}
	for k, v in pairs(_G) do
		if type(k) == "string" and type(v) == "table" and type(v.GetSkillEffect) == "function"
			and type(v.GetTargetArea) == "function" and k ~= "Skill" then
			nomes[#nomes + 1] = k
		end
	end
	table.sort(nomes)
	emit("arma", "total", #nomes)
	local origem, destino = mech:GetSpace(), alvo:GetSpace()
	for _, n in ipairs(nomes) do
		saida:write("# arma ", n, "\n")
		local arma = _G[n]
		local area = tenta(arma.GetTargetArea, arma, origem)
		emit("arma", n .. ".area", area == "ERRO" and area or (area == nil and "nil" or lista_pontos(area)))
		local fx = tenta(arma.GetSkillEffect, arma, origem, destino)
		if fx == "ERRO" or fx == nil then
			emit("arma", n .. ".efeito", tostring(fx))
		else
			emit("arma", n .. ".n", fx.effect:size() .. "," .. fx.q_effect:size())
			for i = 1, fx.effect:size() do
				local d = fx.effect:index(i)
				emit("arma", n .. ".e" .. i .. ".loc", ponto(d.loc))
				for _, c in ipairs(CAMPOS) do
					emit("arma", n .. ".e" .. i .. "." .. c, tostring(d[c]))
				end
			end
			for i = 1, fx.q_effect:size() do
				local d = fx.q_effect:index(i)
				emit("arma", n .. ".q" .. i .. ".loc", ponto(d.loc))
				for _, c in ipairs(CAMPOS) do
					emit("arma", n .. ".q" .. i .. "." .. c, tostring(d[c]))
				end
			end
		end
	end
end

-- Sorteio de casas. Grava valores de rand() para o Rust partir do mesmo estado.
local function coleta_sorteio()
	if DIF_MODO == "rust" then
		DIF_SINCRONIZA()
	else
		for i = 1, 4 do emit("sincronia", i, random_int(32768)) end
	end
	for i = 1, 6 do
		Board:SpawnPawn(PAWN_FACTORY:CreatePawn("Firefly1"))
		local casas = {}
		local tam = Board:GetSize()
		for x = 0, tam.x - 1 do
			for y = 0, tam.y - 1 do
				if Board:IsSpawning(Point(x, y)) then casas[#casas + 1] = x .. "," .. y end
			end
		end
		emit("sorteio", i, table.concat(casas, " "))
	end
end

local function coleta_jogo()
	emit("jogo", "GetSector", Game:GetSector())
	emit("jogo", "GetTeamTurn", Game:GetTeamTurn())
	emit("jogo", "GetTurnCount", Game:GetTurnCount())
	local energia = Game:GetPower()
	emit("jogo", "energia", energia:GetValue() .. "/" .. energia:GetMax())
	for i = 0, 4 do
		emit("jogo", "IsIslandUnlocked" .. i, tostring(Game:IsIslandUnlocked(i)))
	end
end

local function coleta_missao(mapa)
	emit("missao", "mapa", mapa)
	coleta_jogo()
	coleta_casas("casa")
	coleta_caminhos()
	local mech = PAWN_FACTORY:CreatePawn("PunchMech")
	local alvo = PAWN_FACTORY:CreatePawn("Firefly1")
	local l1 = Board:AddPawn(mech, Point(3, 5))
	local l2 = Board:AddPawn(alvo, Point(3, 6))
	emit("entrada", "casas", ponto(l1) .. " " .. ponto(l2))
	emit("entrada", "mech_vivo", tostring(Board:IsPawnAlive(mech:GetId())))
	emit("entrada", "mech_casa", ponto(Board:GetPawnSpace(mech:GetId())))
	emit("entrada", "n_jogador", Board:GetPawns(TEAM_PLAYER):size())
	emit("entrada", "n_inimigo", Board:GetPawns(TEAM_ENEMY):size())
	emit("entrada", "GetEnemyCount", Board:GetEnemyCount())
	for _, t in ipairs({ TEAM_PLAYER, TEAM_ENEMY, TEAM_NONE, TEAM_MECH }) do
		emit("entrada", "GetPawnCount" .. t, Board:GetPawnCount(t))
	end
	for _, n in ipairs({ "Firefly", "PunchMech", "Punch" }) do
		emit("entrada", "GetPawnCount_" .. n, Board:GetPawnCount(n))
	end
	emit("entrada", "GetMechDamage", Board:GetMechDamage())
	emit("entrada", "GetBuildings", lista_pontos(Board:GetBuildings()))
	for _, e in ipairs({ "EVENT_REPAIR_PICKUP", "EVENT_REPAIR_UNDO", "EVENT_ACID_DESTROYED", "EVENT_SPAWNBLOCKED", "EVENT_ENEMY_KILLED" }) do
		emit("entrada", "GetEventCount." .. e, Game:GetEventCount(_G[e]))
	end
	emit("entrada", "time_casa", Board:GetPawnTeam(Point(3, 6)))
	for _, k in ipairs({ "GetMoveSpeed", "GetPathProf" }) do
		emit("entrada", "mech_" .. k, mech[k](mech))
	end
	coleta_casas("com_unidades")
	SetPawn(mech)
	coleta_armas(mech, alvo)
	coleta_sorteio()
	-- depois do sorteio, o rand() segue sincronizado
	local u1 = Board:AddUniqueBuilding("str_bar1")
	local u2 = Board:AddUniqueBuilding("str_power1")
	emit("unico", "casas", ponto(u1) .. " " .. ponto(u2))
	for _, u in ipairs({ u1, u2 }) do
		emit("unico", ponto(u) .. ".vida", Board:GetHealth(u))
		emit("unico", ponto(u) .. ".IsPowered", tostring(Board:IsPowered(u)))
	end
	Board:MarkSpaceDesc(Point(0, 0), "supervolcano")
	emit("unico", "crackable_supervolcano", tostring(Board:IsCrackable(Point(0, 0))))

	-- mudanças de casa e de unidade, lidas logo depois
	local function estado(chave, p)
		emit("mutacao", chave, Board:GetTerrain(p) .. "," .. tostring(Board:IsAcid(p)) .. "," .. tostring(Board:IsFire(p)) .. ","
			.. tostring(Board:IsSmoke(p)) .. "," .. tostring(Board:IsCracked(p)) .. "," .. tostring(Board:IsFrozen(p)) .. ","
			.. Board:GetHealth(p) .. "," .. tostring(Board:IsSafe(p)))
	end
	for _, p in ipairs({ Point(6, 6), Point(6, 7), Point(7, 6), Point(7, 7), Point(1, 1) }) do
		local k = ponto(p)
		estado(k .. ".antes", p)
		Board:SetAcid(p, true) estado(k .. ".acido", p)
		Board:SetSmoke(p, true, false) estado(k .. ".fumaca", p)
		Board:SetCracked(p, true) estado(k .. ".rachada", p)
		Board:SetFrozen(p, true) estado(k .. ".congelada", p)
		Board:RemoveShield(p) estado(k .. ".sem_escudo", p)
		Board:SetSmoke(p, false, false) Board:SetAcid(p, false) estado(k .. ".limpa", p)
	end
	mech:SetFrozen(true)
	emit("mutacao", "mech_congelado", tostring(mech:IsFrozen()) .. "," .. tostring(mech:IsFire()))
	emit("mutacao", "mech_casa_congelada", tostring(Board:IsFrozen(mech:GetSpace())))
	mech:SetFrozen(false)
	emit("mutacao", "mech_descongelado", tostring(mech:IsFrozen()))
	emit("fim", "missao", "ok")
end

---------------------------------------------------------------------- preparação da missão
-- Antes de BaseStart, o motor dá vida aos prédios, sorteia terreno e chuva (BoardPlayer::LoadMission), com chances
-- vindas destas funções Lua. Cada chamada grava argumentos e retorno. Antes dela, 4 valores de rand() deixam o Rust
-- conferir se está no mesmo estado e, se não estiver, voltar a ele. O nome do mapa é tentado por AddMap e pela troca
-- da global do mapa; na 1.2.93 nenhum dos dois funcionou ("?0"), e o comparador acha o mapa pelo terreno.
local ultimo_mapa = "?"
local n_preparo = 0
-- Board::Load executa maps/<nome>.map de novo: a global do mapa escolhido passa a ser outra tabela.
local tabelas_mapa = {}

local function guarda_mapas()
	if MAP_LIST == nil then return end
	for _, nome in ipairs(MAP_LIST) do tabelas_mapa[nome] = rawget(_G, nome) end
end

local function mapa_recarregado()
	local achados = {}
	for nome, t in pairs(tabelas_mapa) do
		if rawget(_G, nome) ~= t then achados[#achados + 1] = nome end
	end
	if #achados == 1 then return achados[1] end
	return "?" .. #achados
end

local function marca_preparo()
	n_preparo = n_preparo + 1
	local k = string.format("%02d", n_preparo)
	if DIF_MODO == "rust" then
		DIF_SINCRONIZA_PREPARO(k)
	else
		local v = {}
		for i = 1, 4 do v[i] = random_int(32768) end
		emit("sincronia_preparo", k, table.concat(v, " "))
	end
	return k
end

local function valores(t)
	if type(t) ~= "table" then return tostring(t) end
	local r = {}
	for i, v in ipairs(t) do r[i] = tostring(v) end
	return table.concat(r, ",")
end

-- maps/maphelper.lua pode ser executado de novo depois da coleta: o envoltório é refeito quando some.
local dif_addmap
local function envolve_addmap()
	if AddMap == nil or AddMap == dif_addmap then return end
	local original = AddMap
	dif_addmap = function(nome)
		ultimo_mapa = nome
		return original(nome)
	end
	AddMap = dif_addmap
end

local function envolve_preparo()
	envolve_addmap()
	local building, env, crack, rain, call = getBuildingHealth, getEnvironmentChance, getCrackChance, getRainChance, CallMethod
	function getBuildingHealth(setor)
		local k = marca_preparo()
		if ultimo_mapa == "?" then ultimo_mapa = mapa_recarregado() end
		-- não comparado: no Rust o nome vem do comparador
		emit("missao", "mapa_preparo", ultimo_mapa)
		local r = building(setor)
		emit("preparo", k .. ".getBuildingHealth", tostring(setor) .. "=" .. valores(r))
		return r
	end
	function getEnvironmentChance(setor, tipo)
		local k = marca_preparo()
		local r = env(setor, tipo)
		emit("preparo", k .. ".getEnvironmentChance", tostring(setor) .. "," .. tostring(tipo) .. "=" .. tostring(r))
		return r
	end
	function getCrackChance(setor)
		local k = marca_preparo()
		local r = crack(setor)
		emit("preparo", k .. ".getCrackChance", tostring(setor) .. "=" .. tostring(r))
		return r
	end
	function getRainChance(setor)
		local k = marca_preparo()
		local r = rain(setor)
		emit("preparo", k .. ".getRainChance", tostring(setor) .. "=" .. tostring(r))
		return r
	end
	function CallMethod(obj, func, a1, a2)
		if func ~= "IsCracksAllowed" then return call(obj, func, a1, a2) end
		local r = call(obj, func, a1, a2)
		emit("preparo", "IsCracksAllowed", tostring(obj) .. "=" .. tostring(r))
		emit("preparo", "missao_id", tostring(tenta(function() return GAME:GetMission(obj).ID end)))
		return r
	end
end

coleta_globais()
coleta_valores()
coleta_rand()
coleta_dano()
emit("fim", "boot", "ok")

-- `CreatePawn` derruba o jogo durante a inicialização: as unidades esperam o menu. O gancho é uma função
-- que o motor chama ao desenhar textos.
local function fim_do_menu()
	envolve_addmap()
	guarda_mapas()
	coleta_unidades()
	emit("fim", "menu", "ok")
	if DIF_MODO == "boot" then
		saida:close()
		os.exit(0)
	end
end

if DIF_MODO == "rust" then
	fim_do_menu()
	if DIF_MAPA then
		envolve_preparo()
		DIF_COLETA_MISSAO = function()
			coleta_missao(DIF_MAPA)
			saida:close()
		end
	else
		saida:close()
	end
else
	if DIF_MODO == "missao" then
		envolve_preparo()
		-- A coleta roda quando a missão começa. O motor escolhe o mapa: o comparador o identifica pelo terreno.
		Mission.BaseStart = function(self)
			saida:write("# missao: BaseStart\n")
			for k, v in pairs(_G) do
				if type(k) == "string" and v == getmetatable(self) then saida:write("# tipo da missao: ", k, "\n") end
			end
			coleta_missao("real")
			saida:close()
			os.exit(0)
		end
	end
	local get_text = GetText
	local chamadas = 0
	function GetText(...)
		envolve_addmap()
		chamadas = chamadas + 1
		if chamadas == 100 then
			saida:write("# menu: GetText chamada 100 vezes\n")
			fim_do_menu()
		end
		return get_text(...)
	end
end
