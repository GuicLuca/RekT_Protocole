import os
import json
import glob
from collections import defaultdict
import re


def get_topic_name_from_id(id_number):
    """Convertit un ID en son nom de topic correspondant"""
    # L'ID est sous la forme [player_id][topic_number]
    # Par exemple, 15 signifie player_id=1 et topic_number=5 (PlayerName)
    if id_number < 10:  # Cas où l'ID est trop petit
        return f"Unknown_ID_{id_number}"

    # Extraire le player_id (premier chiffre) et le topic_number
    str_id = str(id_number)
    player_id = int(str_id[0])
    topic_number = int(str_id[1:])

    # Mapper les topic_numbers aux noms
    topic_names = {
        1: f"PlayerScore_P{player_id}",
        2: f"PlayerHealth_P{player_id}",
        3: f"PlayerColor_P{player_id}",
        4: f"PlayerTitle_P{player_id}",
        5: f"PlayerName_P{player_id}",
        6: f"PlayerPosition_P{player_id}",
        7: f"PlayerRotation_P{player_id}",
        8: f"PlayerIsInTeam_P{player_id}",
        9: f"PlayerStats_P{player_id}",
        10: f"PlayerLastUpdated_P{player_id}"
    }

    return topic_names.get(topic_number, f"Unknown_Topic_{player_id}_{topic_number}")


def get_base_topic_name(full_topic_name):
    """Extrait le nom de base du topic sans le suffixe _PX"""
    match = re.match(r"(.*?)_P\d+$", full_topic_name)
    if match:
        return match.group(1)
    return full_topic_name


def analyser_fichiers_json(dossier):
    # Liste des cartes à traiter
    map_names = ["clear", "wall", "glass"]

    for map_name in map_names:
        print(f"\n===== Analyse des fichiers pour la carte '{map_name}' =====")

        # Chercher tous les fichiers JSON qui contiennent le nom de la carte
        pattern = os.path.join(dossier, f"*{map_name}*.json")
        fichiers = glob.glob(pattern)

        if not fichiers:
            print(f"Aucun fichier JSON trouvé pour la carte '{map_name}'")
            continue

        print(f"Nombre de fichiers trouvés: {len(fichiers)}")

        # Dictionnaire pour stocker les sommes et compteurs pour chaque clé
        donnees_combinees = defaultdict(lambda: {"somme": 0, "compteur": 0})

        # Dictionnaire pour les statistiques globales par type de topic
        stats_globales = defaultdict(lambda: {"somme": 0, "compteur": 0})

        # Parcourir tous les fichiers JSON
        for fichier in fichiers:
            try:
                with open(fichier, 'r', encoding='utf-8') as f:
                    donnees = json.load(f)

                # Traiter le format spécifique [clé, valeur]
                if isinstance(donnees, list):
                    for item in donnees:
                        # Vérifier si l'élément est une paire [clé, valeur]
                        if isinstance(item, list) and len(item) == 2:
                            cle_originale = item[0]
                            valeur = item[1]

                            # Convertir la clé si c'est un "Data::XXXX"
                            if cle_originale.startswith("Data::"):
                                # Extraire le nombre après "Data::"
                                match = re.match(r"Data::(\d+)", cle_originale)
                                if match:
                                    # Convertir en entier et retirer le bit de poids fort (bit 63)
                                    nombre = int(match.group(1))
                                    nombre_original = nombre & ~(1 << 63)  # Désactive le bit 63

                                    # Traduire l'ID en nom de topic
                                    nom_topic = get_topic_name_from_id(nombre_original)
                                    cle = nom_topic  # Retire le préfixe "Data::"

                                    # Obtenir le type de base du topic (sans _PX)
                                    base_topic = get_base_topic_name(nom_topic)

                                    # Vérifier si la valeur est numérique
                                    if isinstance(valeur, (int, float)):
                                        donnees_combinees[cle]["somme"] += valeur
                                        donnees_combinees[cle]["compteur"] += 1

                                        # Ajouter aux statistiques globales par type
                                        stats_globales[base_topic]["somme"] += valeur
                                        stats_globales[base_topic]["compteur"] += 1
                            else:
                                # Pour les clés qui ne commencent pas par "Data::"
                                cle = cle_originale
                                if isinstance(valeur, (int, float)):
                                    donnees_combinees[cle]["somme"] += valeur
                                    donnees_combinees[cle]["compteur"] += 1
                        else:
                            print(f"Format inattendu dans {fichier}: {item}")
                else:
                    print(f"Format de données non pris en charge dans {fichier}")

            except Exception as e:
                print(f"Erreur lors de la lecture du fichier {fichier}: {e}")

        # Calculer et afficher les moyennes par clé individuelle
        print("\nMoyennes calculées par clé individuelle:")
        print("-" * 50)
        for cle, info in sorted(donnees_combinees.items()):
            if info["compteur"] > 0:
                moyenne = info["somme"] / info["compteur"]
                print(f"{cle}: {moyenne:.4f} (calculé à partir de {info['compteur']} valeurs)")
            else:
                print(f"{cle}: Aucune valeur numérique trouvée")

        # Afficher les statistiques globales par type de topic
        print("\nStatistiques globales par type de topic:")
        print("-" * 50)
        for base_topic, info in sorted(stats_globales.items()):
            if info["compteur"] > 0:
                moyenne = info["somme"] / info["compteur"]
                print(f"{base_topic}:")
                print(f"  Somme totale: {info['somme']}")
                print(f"  Nombre de valeurs: {info['compteur']}")
                print(f"  Moyenne: {moyenne:.4f}")
                print()
            else:
                print(f"{base_topic}: Aucune valeur numérique trouvée")
                print()


def main():
    # Demander le chemin du dossier à l'utilisateur
    dossier = input("Veuillez entrer le chemin du dossier contenant les fichiers JSON à analyser: ")

    # Vérifier si le dossier existe
    if not os.path.isdir(dossier):
        print(f"Erreur: Le dossier '{dossier}' n'existe pas ou n'est pas accessible.")
        return

    # Analyser les fichiers JSON
    analyser_fichiers_json(dossier)


if __name__ == "__main__":
    main()