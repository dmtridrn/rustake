use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustake")]
#[command(about = "Make en rust", long_about = None)] 
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>, //option pcq si ya pas de souscommande (None) on fait le build
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialise un nouveau projet avec un Rustakefile.toml
    Init,
    /// Nettoie les fichiers de build et le cache
    Clean,
}