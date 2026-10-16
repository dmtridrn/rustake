use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    pub empreintes: HashMap<String, String>, //meme logique que les blocs, la on a (Nom,Hash)
}

impl Cache {

    ///Renvoie un nouvel élément vide de struct Cache
    pub fn new() -> Self {
        Self {
            empreintes: HashMap::new(),
        }
    }

    ///Charge le cache demandé et renvoie un nouvel élément Cache avec empreintes rempli
    pub fn charger() -> Self {
        let chemin = ".rustake/cache"; //exemple du sujet on pourra changer         
        let resultat = fs::read_to_string(chemin);
        
        match resultat {
            Ok(contenu) => {
                //si il est deja la on prend le contenu texte -> struct Cache (deserialise)
                match toml::from_str(&contenu) {
                    Ok(c) => c,
                    Err(_) => Self::new(), //sinn on crée
                }
            },
            Err(_) => Self::new(), // si y pas de fichier on part de 0
        }
    }
    ///on écrit le contenu du hashmap dans le fichier (et on le crée s'il est pas la)
    pub fn sauvegarder(&self) -> Result<(), String> {
        if let Err(_) = fs::create_dir_all(".rustake") { //create_dir_all ca crée l'arborecense si elle existe pas deja
            return Err("Impossible de créer le dossier .rustake".to_string());
        }

        // on serialise (struct -> texte)
        match toml::to_string(self) {
            Ok(texte) => {
                if let Err(_) = fs::write(".rustake/cache", texte) {
                    return Err("Impossible d'écrire le fichier de cache".to_string());
                }
                Ok(())
            },
            Err(_) => Err("Erreur de transformation du cache".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialisation_deserialisation() {
        let mut cache = Cache::new();
        cache.empreintes.insert("fichier.rs".to_string(), "12345hash".to_string());

        //on check si serialisation -> déserialisation donne le même resultat 
        let texte = toml::to_string(&cache).unwrap();
        let cache_restaure: Cache = toml::from_str(&texte).unwrap();

        assert_eq!(cache_restaure.empreintes.get("fichier.rs").unwrap(), "12345hash");
    }
}