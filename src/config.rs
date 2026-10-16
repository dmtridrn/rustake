use serde::Deserialize; //passer du texte a struct
use std::collections::HashMap;
use std::fs;


#[derive(Debug, Deserialize, Clone)]
pub struct Target { //chaque bloc [nom] du fichier toml
    #[allow(dead_code)] //temporaire pr éviter le waring unused ct insuportable
    pub depends: Vec<String>, // les dépendances
    pub command: String,      // la commande
}

#[derive(Debug, Deserialize)]
pub struct Configuration { 
    //dico (nom,struct) donc t'as tout le fichier toml la dedans
    pub targets: HashMap<String, Target>,
}

impl Configuration {
    ///lit le fichier et renvoie la config si tout est ok 
    ///séparé en 2 fonction pour faire un test qui lit pas vraiment le fichier juste il teste le parsing
    pub fn parser_texte(contenu: &str) -> Result<(Self, Vec<String>), String>{
        //la crate toml parse elle-même et remplis la struct marquée par Deserialized, ici Bloc
        let temp = toml::from_str(contenu); 
        let config_finale : Configuration = match temp { //verification que le parsing s'est bien fait
            Ok(config) => config,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        //On récupère les outputs (donc run, test etc) pour en avoir la liste dans le main
        let outputs = config_finale.targets.keys().cloned().collect();
        
        //On renvoie la config finale et les cibles
        Ok((config_finale, outputs))   
    }

    ///charge le fichier Rustakefile.toml et renvoie une erreur si il y a un probleme
    pub fn charger() -> Result<(Self, Vec<String>),String>{
        let nomfic: &str = "Rustakefile.toml";
        let res = fs::read_to_string(nomfic);
        let contenu = match res{ //check pr le read
            Ok(texte) => texte,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        Self::parser_texte(&contenu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_texte_valide() {
        // format toml
        let toml_texte = r#"
        [targets.compilation]
        depends = ["src/main.rs"]
        command = "rustc main.rs"
        "#;

        // on parse
        let (config, outputs) = Configuration::parser_texte(toml_texte).unwrap();

        // on check si targets contient bien "compilation"
        assert!(config.targets.contains_key("compilation"));
        
        // on check si les commandes/dépendances sont les bonnes
        let cible = config.targets.get("compilation").unwrap();
        assert_eq!(cible.command, "rustc main.rs");
        assert_eq!(cible.depends, vec!["src/main.rs"]);

        assert!(outputs.contains(&"compilation".to_string()));
    }

    #[test]
    fn test_parser_texte_invalide() {
        //texte random pas au format toml
        let toml_texte = "bonjour les test =/=/aAfegr ceci n'est pas du toml";
        let resultat = Configuration::parser_texte(toml_texte);
        assert!(resultat.is_err());
    }
}