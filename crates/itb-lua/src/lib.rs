//! Host Lua 5.1 do Into the Breach em Rust.
//!
//! Estratégia (ver docs/ARQUITETURA.md): o jogo original é um motor C++ + ~150 scripts Lua.
//! Em vez de traduzir o EXE inteiro, reimplementamos em Rust a API nativa que os scripts
//! chamam (especificada por `api_manifest`, gerado do EXE) e executamos os scripts ORIGINAIS
//! da instalação do usuário, sem modificá-los e sem redistribuí-los.

pub mod api_manifest;
pub mod game;
pub mod host;
pub mod map;
pub mod pawns;
pub mod records;
pub mod skill;
pub mod world;
pub mod values;

pub use host::{Host, HostConfig, Outcome, Report, Status};
