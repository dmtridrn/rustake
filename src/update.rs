use std::fs::File;
use std::io::Read;
use blake3::Hasher;


///point d'entrée pour calculer le hash, ouvre le fichier si possible et appelle to_hash pour récuperer le hash du fichier
pub fn calculer_hash_chemin(chemin: &str) -> Result<String, String> {
    match File::open(chemin) {
        Ok(fichier_ouvert) => {
            to_hash(fichier_ouvert)
        },
        Err(_) => Err(format!("Impossible d'ouvrir le fichier : {}", chemin)),
    }
}

///récupère un fichier et renvoie son hash si pas de problème, sinon renvoie le texte e de l'erreur sous format Err(e)
fn to_hash(mut file : File)-> Result<String, String>{
    let mut hasher= Hasher::new();
    let mut content =  [0; 1024];

    loop {
        match file.read(&mut content) {
            Ok(0) => break,
            Ok(count) => {
                hasher.update(&content[..count]);
            },
            Err(e) => return Err(format!("Erreur de lecture : {}", e)), //Err au lieu de except ca va faire plaisir au degs
        }
    }
    let h = hasher.finalize();
    Ok(h.to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_calculer_hash_coherence() {
        let nom_fichier = "test_hash.txt";
        
        // fichier temporaire
        let mut file = File::create(nom_fichier).unwrap();
        file.write_all(b"hehe c'est le test la ou quoi").unwrap();//write all écrit tout le contenu du buffer
        
        // on calcule le hash 2 fois 
        let hash1 = calculer_hash_chemin(nom_fichier).unwrap();
        let hash2 = calculer_hash_chemin(nom_fichier).unwrap();
        
        // c'est censé être le même
        assert_eq!(hash1, hash2);

        std::fs::remove_file(nom_fichier).unwrap();
    }

    #[test]
    fn test_fichier_inexistant() {
        let res = calculer_hash_chemin("fichier_qui_n_existe_pas.txt");
        assert!(res.is_err());
    }
}