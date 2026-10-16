use std::process::Command;
use std::fs;
use std::path::Path;

#[test]
fn test_commande_init() {
    // on prépare un espace safe où tester
    let dossier_test = "test_integration_init";
    let _ = fs::remove_dir_all(dossier_test);
    fs::create_dir(dossier_test).unwrap();

    //on lance le prog comme un utilisateur qui écrit cargo run -- init
    let status = Command::new("cargo")
        .args(["run", "--", "init"])
        .current_dir(dossier_test) //vive cette commande on se place dans notre espace safe
        .status()
        .expect("Erreur lors de l'appel à cargo");

    //check pour une erreur
    assert!(status.success());

    // check si le fichier est bien la
    let chemin_toml = format!("{}/Rustakefile.toml", dossier_test);
    assert!(Path::new(&chemin_toml).exists());

    fs::remove_dir_all(dossier_test).unwrap();//supprime cette ligne stv voir ce que ca donne et pas seulement le résultat du test
}