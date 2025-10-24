import subprocess
import time
import sys
import os

SERVER_PATH = r"D:\Dev\Rust\RekT_Protocole\RektBroker"
CLIENT_PATH = r"D:\Dev\Rust\RekT_Protocole\Client"

def run_experiment(map_name):

    print(f"--- Démarrage du serveur avec la carte {map_name} ---")
    server_process = subprocess.Popen(f"cargo run --features profiling -- {map_name}", shell=True, cwd=SERVER_PATH)
    time.sleep(1)

    client_processes = []
    for player_id in range(1, 5):
        print(f"Lancement du client {player_id} sur la carte {map_name}")
        client_process = subprocess.Popen(f"cargo run -- {map_name} {player_id}", shell=False, cwd=CLIENT_PATH)
        client_processes.append(client_process)


    server_process.wait()


    print("Serveur terminé, attente de 2 secondes...")
    time.sleep(2)

    for process in client_processes:
        if process.poll() is None:  # Si le processus est toujours en cours
            process.terminate()


def main():
    map_names = ["clear", "wall", "glass"]

    for cycle in range(1, 6):
        print(f"=== Cycle d'expériences {cycle}/5 ===")

        for map_name in map_names:
            print(f"== Expérience avec la carte {map_name} ==")
            run_experiment(map_name)


if __name__ == "__main__":
    try:
        main()
        print("Toutes les expériences ont été complétées avec succès!")
    except KeyboardInterrupt:
        print("Interruption manuelle des expériences.")
        sys.exit(1)
    except Exception as e:
        print(f"Erreur lors de l'exécution des expériences : {e}")
        sys.exit(1)
