//! Núcleo de regras do Into the Breach em Rust (sem IO, sem Lua, sem SDL).
//!
//! `board` é reconstrução semântica amparada pela análise do Breach.exe.
//! `input` é código NOVO, projetado para o motor Rust; não foi decompilado.

pub mod board;
pub mod input;

/// Consultas nativas reconstruídas em Rust (Etapa 03).
pub mod board_queries;
/// Adaptador NOVO de intenções de entrada para o tabuleiro (Etapa 03).
pub mod interaction;

/// Ocupação por unidades, bloqueio de caminho e acessores de Pawn (Etapa 05).
pub mod board_occupancy;

/// Tipos de valor Point/GL_Color e conversões numéricas idênticas ao original (Etapa 06).
pub mod geometry;
/// rand()/srand() da CRT MSVC e random_int/random_bool (Etapa 06).
pub mod rng;
/// Tamanho virtual de tela e escalas de UI (Etapa 06).
pub mod display;
/// Estruturas de UI (ButtonDesc...) (Etapa 06).
pub mod ui;
/// Passagem e alcance em linha reta (Etapa 09).
pub mod pathing;
/// Consultas de terreno: IsTerrain, IsCrackable, IsPod, IsItem, IsDangerous, IsSpawning (Etapa 10).
pub mod board_terrain;
/// Dano letal: IsDeadly / WouldDie (Etapa 10).
pub mod damage;
/// Alcance (Dijkstra) e caminho (A*) (Etapa 10).
pub mod pathfinding;
/// Aplicação de SkillEffect ao tabuleiro: dano, empurrão, fogo, fumaça, ácido, gelo (Etapa 11).
pub mod apply;
/// Executor de SkillEffect com relógio: fila do tabuleiro, atrasos, empurrões e projéteis (Etapa 12).
pub mod executor;
/// Mapa de missão aplicado ao tabuleiro (Etapa 13).
pub mod map;
/// Criação de unidades e entrada no tabuleiro (Etapa 13).
pub mod pawn_factory;
/// Estado de `GameMap` lido pelos scripts (Etapa 13).
pub mod game;
