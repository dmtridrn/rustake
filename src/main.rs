mod cli;
mod config;
mod update;
mod cache;
mod graph;

use cli::{Cli, Commands};
use std::fs;
use std::process::Command;
use config::Configuration;
use clap::Parser;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

/// Début de rustake.
/// Parse les arguments de la ligne de commande et exécute la commande appropriée :
/// - init : Crée un fichier Rustakefile.toml par défaut
/// - clean : Supprime les cibles et le cache
/// - None (par défaut) : Lance le processus de build rustake
fn main() {
    //recuperation des arguments
    let args = Cli::parse(); //remplace env.get
    match args.command {
        //On vérifie ce que la personne a demandé (init, clean ou rien ce qui signifie qu'on fait rustake)
        Some(Commands::Init) => init(),
        
        // rustake clean
        Some(Commands::Clean) => clean(),
        
        // rustake
        None => rustake(),
    }

}



///créé un Rustakefile.toml par défaut
///si problème, ecrit l'erreur dans la sortie erreur
fn init(){
    let init_path = Path::new("Rustakefile.toml");
    //S'il existe déjà, on ne recréé pas le fichier et on indique l'erreur à l'utilisateur
    if init_path.exists(){
        eprintln!("Le fichier existe déjà");
        return;
    }
    //On écrit dans le .toml
    let file: Result<(), std::io::Error> = fs::write(init_path, 
    r#"[targets.program]
    depends = ["src/main.rs", "src/lib.rs"]
    command = "rustc src/main.rs -o program"
    
    [targets.test]
    depends = ["src/lib.rs", "tests/test.rs"]
    command = "cargo test"

    [targets.run]
    depends = ["program"]
    command = "./program"
    "#); //r# pour raw string donc pas besoin de \ partout

    match file{
        Ok(_o) => println!("Fichier créé"),
        Err(e) => eprintln!("{}", e),
    }
}


///Supprime les cibles et le cache
fn clean() {
    //on charge le toml et s'il existe on nettoie
    match Configuration::charger() {
        Ok((config, _)) => {
            for cible in config.targets.keys() {
                if Path::new(cible).exists() { //on vérifie s'il y a bien le fichier pour éviter les erreurs
                    match fs::remove_file(cible) {
                        Ok(_) => println!("fichier supprimé : {}", cible),
                        Err(e) => eprintln!("{} n'a pas pu etre supprimé car erreur : {}", cible, e),
                    }
                }
            }
        },
        Err(_) => eprintln!("erreur dans le chargement de la configuration")
    }

    //on supprime le cache
    if Path::new(".rustake").exists() {
        match fs::remove_dir_all(".rustake") {
            Ok(_) => println!("cache nettoyé"),
            Err(e) => eprintln!("erreur nettoyage cache : {}", e),
        }
    }
    
}




/// Exécute rustake
/// Cette fonction :
/// Charge la configuration et détecte les dépendances cycliques via un tri topologique.
/// Charge le cache d'empreintes digitales (hashes) des fichiers sources.
/// Exécute les cibles niveau par niveau en parallèle (threads) si nécessaire.
/// Met à jour le cache et le sauvegarde à la fin.
fn rustake() {
    // on charge le fichier .toml
    let (config, outputs) = match Configuration::charger() {
        Ok((c, o)) => (c, o),
        Err(e) => { eprintln!("{}", e); return; }
    };

    let ordre_execution = match graph::tri_topologique(&config.targets) {
        Ok(ordre) => ordre,
        Err(e) => {eprintln!("{}", e); return;}
    };

    let targets = config.targets.clone();
    // on récupère les anciens hash
    let cache = Arc::new(Mutex::new(cache::Cache::charger()));
    let command_changed = Arc::new(Mutex::new(Vec::<String>::new()));

    // parcours de chaque "niveau" du tri topologique
    for niveau in ordre_execution{
        let mut threads = Vec::new();

        for cible in niveau{
            println!("on fait {cible}");
            let cible_clone = cible.clone();
            let outputs_clone = outputs.clone();

            //Versions thread safe du cache et du tableau command_changed
            let cache_clone = Arc::clone(&cache);
            let command_changed_clone = Arc::clone(&command_changed);

            let detail_clone = if let Some(x) = targets.get(&cible){ x.clone()}
                else{
                    eprintln!("Cible {} introuvable", &cible); return
                };

            let thr = thread::spawn(move || -> Result<(), String> {

                if is_needing_rebuild(&cible_clone, &detail_clone, &outputs_clone, &cache_clone, &command_changed_clone) {
                    println!("ça build [{}] : {}", cible_clone, detail_clone.command);

                    let (shell, flag) = if cfg!(target_os = "windows") { 
                        ("cmd", "/C") 
                    } else { 
                        ("sh", "-c") 
                    };
                    //on lance une commande, et s'il y a erreur, la convertit en String pour pouvoir gérer ça dans le match
                    let status = Command::new(shell)
                        .arg(flag)
                        .arg(&detail_clone.command)
                        .status()
                        .map_err(|e| e.to_string())?;

                    match status {
                        s if s.success() => {
                            {
                                let mut cmd = command_changed_clone.lock().unwrap();
                                if !cmd.contains(&cible_clone) {
                                    cmd.push(cible_clone.clone());
                                }
                            }

                            let mut cache_lock = cache_clone.lock().unwrap();

                            if let Ok(h) = update::calculer_hash_chemin(&cible_clone) {
                                cache_lock.empreintes.insert(cible_clone.clone(), h);
                            }
                            for fichier in &detail_clone.depends {
                                if !outputs_clone.contains(fichier) {
                                    if let Ok(h_source) = update::calculer_hash_chemin(fichier) {
                                        cache_lock.empreintes.insert(fichier.clone(), h_source);
                                    }
                                }
                            }
                            Ok(())
                        }
                        _ => Err(format!("erreur: {} a échoué", cible_clone))
                        }
                } else {
                    println!("'{}' a pas changé", cible_clone);
                    Ok(())
                }
            });

            threads.push(thr);
        }

        // attendre tous les threads du niveau
        for thr in threads {
            match thr.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    eprintln!("{}", e);
                    //On exit dans le join pour eviter de tt casser
                    std::process::exit(1);
                }
                Err(_) => {
                    eprintln!("thread panic");
                    std::process::exit(1);
                }
            }
        }
    }

    // sauvegarde finale du cache
    let cache_final = cache.lock().unwrap();
    if let Err(e) = cache_final.sauvegarder() {
        eprintln!("err save du cache: {}", e);
    }
}

///récupère une cible et vérifie si elle a besoin d'être build de nouveau (ou pour la première fois) 
fn is_needing_rebuild(cible : &String, detail : &config::Target, outputs : &Vec<String>, cache: &Arc<Mutex<cache::Cache>>, command_changed: &Arc<Mutex<Vec<String>>>) -> bool{
    if !Path::new(&cible).exists() { //s'il n'y pas le fichier on build normal
            return true;
        }

        for fichier in &detail.depends {
            //Si c'est bien un fichier et pas une cible
            if outputs.contains(fichier){
                let cmd = command_changed.lock().unwrap();
                if cmd.contains(fichier) { //si une dépendance a bougé on bouge celui la
                    return true;
                }
            }
            else{
                // hash actuel
                let hash_actuel = match update::calculer_hash_chemin(fichier) {
                    Ok(h) => h,
                    Err(e) => { eprintln!("{}", e); continue; }
                };

                // on vérifie s'il y a deja un hash et on le compare si c'est le cas
                //s'il a changé ou s'il est nouveau on executera la commande
                let cache_locked = cache.lock().unwrap();
                match cache_locked.empreintes.get(fichier) {
                    Some(vieux_hash) => {
                        if vieux_hash != &hash_actuel {
                            println!("{} a changé ", fichier);
                            return true;
                        }
                    },
                    None => {
                        println!("nouveau fichier : {}", fichier);
                        return true;
                    }
                }
            }

        }
        false

}