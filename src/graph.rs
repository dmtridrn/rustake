use crate::config::Target;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::collections::HashMap;

//On implémente le graphe des dépendances et cibles pour ensuite faire un tri topologique 
// et avoir l'ordre d'execution (cas dépendance cyclique pas encore géré et renvoie None dans ce cas pour l'instant)
//On renvoie maintenant un tableau de tableaux pour regrouper, dans l'ordre topologique, les groupes de cibles qu'on peut executer en même temps
///Effectue un tri topologique "par niveau" sur les cibles pour savoir dans quel ordre les exécuter
/// Renvoie une erreur si dépendance circulaire
/// targets : &HashMap<String, Target> le nom de chaque cible et la cible en elle-même
/// return : Result<Vec<Vec<String>>, String> la liste "par niveau" (pour un build multi-threadé)(une sous-liste par niveau) des noms des cibles, 
/// dans l'ordre dans lequel il faut les exécuter 
/// ou une erreur si problème durant le tri
pub fn tri_topologique(targets : &HashMap<String, Target>) -> Result<Vec<Vec<String>>, String>{
    //On initialise le graphe donc dans l'ordre : le degré entrant de chaque element, le graphe en lui-même, et les noeuds
    let mut degre_entrant: HashMap<String, u16> = HashMap::new();
    let mut graphe: HashMap<String, Vec<String>> = HashMap::new();
    let mut noeuds: HashSet<String> = HashSet::new();

    // Initialisation avec les cibles, et pour l'instant, tous les neouds ont un degré 0 par defaut
    for nom_target in targets.keys() {
        degre_entrant.insert(nom_target.to_string(), 0);
        graphe.insert(nom_target.to_string(), Vec::new());
        noeuds.insert(nom_target.to_string());
    }

    // Remplir le graphe et maj des degrés entrants
    for (nom_target, target) in targets {
        // Pour chaque dépendance, on ajoute une arête vers la cible
        for dependances in &target.depends {
            // Si la dépendance est déjà dans le graphe alors on ajoute une arête de dependance à nom_target 
            if noeuds.contains(dependances) {
                graphe.entry(dependances.clone()).or_insert_with(Vec::<String>::new).push(nom_target.to_string());
                
                if let Some(x) = degre_entrant.get_mut(nom_target){
                    *x+=1;
                }
                else{
                    return Err("La cible {nom_target} n'est pas dans le graphe".to_string());
                }
            }
            // Sinon si la dépendance est un fichier source (et pas une cible), on l'ignore pour le 
            // tri car on ne peut pas "exécuter" un fichier source
        }
    }

    //On commence par les nœuds sans dépendances (degré 0)
    let mut queue = VecDeque::new();
    
    for (noeud, degree) in &degre_entrant {
        if *degree == 0 {
            queue.push_back(noeud.clone());
        }
    }

    let mut liste_triee: Vec<Vec<String>> = Vec::new();
    //Pour avoir le compte de combien on en a vu qu'on fait pas
    let mut total_dans_liste =0;

    while !queue.is_empty() {
        let mut l_niveau: Vec<String> = Vec::new();
        let taille_niveau = queue.len();
    
        for _ in 0..taille_niveau {
            let noeud = if let Some(x) =  queue.pop_front(){x} 
                else{ return Err("pile vide alors que la taille de la pile est de {taille_niveau}".to_string())};
            l_niveau.push(noeud.clone());
            total_dans_liste += 1;

            // Pour chaque voisin (element qui dépend de ce noeud), on réduit son degré entrant
            if let Some(voisins) = graphe.get(&noeud) {
                for voisin in voisins {
                    let degre = if let Some(x) = degre_entrant.get_mut(voisin){x} 
                        else{return Err("erreur dans la récupération du degré entrant de {voisin}".to_string())};
                    *degre -= 1;
                    // Si le degré devient 0, le noeud est ajouté à la queue (car toutes ses dépendances 
                    // ont ete mises dans la liste (et donc seront executées avant lui))
                    if *degre == 0 {
                        queue.push_back(voisin.clone());
                    }
                }
            }
        }

        liste_triee.push(l_niveau);
    }

    // Vérification : si le résultat contient moins de nœuds que le total, il y a un cycle 
    if total_dans_liste != targets.len() {
        return Err("Erreur : Dépendance circulaire détectée".to_string());
    }
    Ok(liste_triee)
}