# Rust Media Player -- Soyah Amin - Sauvage Lucas 

- SOYAH Amin (amin.soyah.etu@univ-lille.fr)
- SAUVAGE Lucas (lucas.sauvage.etu@univ-lille.fr) 

**Dépôt du projet de COA pour l'année 2025-2026.** 

## Projet 

### Commandes 

Le projet peut-être compilé avec `cargo run`.
Pour activer les logs, il faut utiliser la commande `RUST_LOG=Audio_Player=debug cargo run`.

La doc peut être générée avec (et ouverte) avec `cargo doc (--open)`. 

Les tests sont exécutés avec la commande `cargo test`.

### Arborescence du projet 

- data_samples (Fichiers audio pour tester l'application sans importer de fichier)
    - 440 Hz (10 seconds of A).mp3 
    - chill_lofi_piano.mp3
    - sample-2.ogg
    - sample-3.wav 
    - test.mp3
- src 
    - fileReader.rs 
    - lib.rs 
    - main.rs 
    - play.rs
    - read.rs 
    - slint_logic.rs 
    - volume.rs 
- tests (les tests de slint_logic sont définis dans un module du fichier)
    - fileReader_tests.rs 
    - volume_tests.rs 
    - play_tests.ts 
- ui (Composant Slint)
    - app-window.slint
- build.rs (Fichier permettant de compiler le composant Slint)
- Cargo.toml (Métadata sur le fichier et dépendences) 
- Cargo.lock (Sous-dépendences) 

## UI 


### Contenu de l'UI

L'interface utilisateur a été créée avec la bibliothèque Slint. 
Le fonctionnement est très simple, chaque bouton déclenche un callback, qui est ensuite récupéré du côté back-end. 
Il est ensuite traité par Rust, comme expliqué plus bas. 

Les élèments de l'UI sont très simplistes : 

Il existe un bouton pour charger un fichier audio provenant du dossier `data_samples` situé à la racine de ce projet, afin de tester rapidement le lecteur. 
Il est également possible d'importer son propre fichier, à condition qu'il soit d'extension mp3, wav ou ogg. 
De plus, l'import copie le fichier de l'utilisateur dans le dossier `data_samples`, afin de pouvoir le ré-utiliser facilement. 

Les deux boutons servent à contrôler la lecture du flux audio. 
Le bouton Stop arrête le flux et ré-initalise la progression du fichier (le prochain Play reprendra au début). 
Le bouton Play/Pause permet d'arrêter temporairement le flux et le reprendre là où il s'était arrêté. 

Il existe un slider qui est utilisé pour le contrôle du volume. 

Finalement, il existe deux élèments permettant d'afficher le titre du fichier séléctionné ainsi que l'avancement dans le morceau (une barre de progression). 

## Rust 

### Canal de communication Multi-Producteurs <-> Simple-Consommateur

La bibliothèque `mpsc` permet de définir un système multi-producteurs simple-consommateur, ce qui est très pratique dans notre cas. 
En effet, chaque callback va posséder son propre producteur afin d'envoyer un message via ce cannal. 
Ensuite, un unique consommateur va ensuite traiter tous les messages envoyés par les producteurs. 
Cette bibliothèque utilise une file, ce qui permet de traiter chaque ordre séquentiellement et également traiter les messages dans l'ordre.
De plus, cette bibliothèque implémente le trait `Clone` pour l'objet `Sender`, ce qui permet, du côté Rust, de donner l'ownership requis par la closure. 

### Pipeline Slint - Rust 

Comme mentionné plus haut, Slint permet de définir des callbacks, qui seront récupérés et traités côté back-end. 
Chaque callback définit permet d'être récupéré via une fonction `<UI Reference>.on_<callback_name>`. 
Ensuite, via le clone du producteur, un message provenant d'une énumération <small>(plus de détails plus bas)</small> est envoyé au consommateur. 

Le consommateur est lui bloqué dans un thread, où il attend de recevoir un message. 
Il appelle ensuite la fonction `UICommandsReceiver::match_command()`, qui grâce à un pattern matching, permet d'effectuer le code lié à la commande.

### Conception d'Objets 

Les objets créés pour la lecture du flux audio peuvent être comparés à des poupées russes. 
Ce système peremt d'éviter les problèmes d'ownership liés à l'encapsulation de réference dans les structures. 

#### FileReader 

[**Source**](./src/fileReader.rs)

`FileReader` est utilisé pour décoder le fichier de type `mp3`, `ogg` ou `wav` et produire des samples f32. 
Grâce à la fonction `new()` on ouvre un fichier audio, en utilisant `symphonia` on détecte le format `probe` et on créer le `decoder` en fonction du format récupéré et remplit le buffer de samples `sample_buf`

Il implémente le trait `Iterator`, ce qui permet d'appeler `next()`. 

Le rôle de next est alors de remplir un buffer avec les caractéristiques des samples. 
Si le buffer est rempli, alors chaque sample est vidé un par un pour être joué. 
Sinon le buffer est rempli. 

Cela permet d'éviter de charger tout le fichier, puis de le jouer. 
En effet cette méthode introduisait des temps d'attentes énormes (environ 15 secondes pour un fichier de 3minutes). 

#### Volume 

[**Source**](./src/volume.rs)

Le volume est utilisé comme `wrapper` de FileReader qui permet de modifier le volume en temps réel sur les samples audio. 
`Volume` contient:
- Un `FileReader` qui fournit les samples audio
- Un `Arc<Mutex<f32>>` qui représente le volume partagé entre les composants du programmes.
Le type `Arc<Mutex<f32>>` nous permet effectivement de partager le volume entre les différents threads de notre programme; l'`UI` et le thread `audio` tout en le modifiant en temps réel.

Dans ce fichier on implémente également le trait Iterator, ce qui permet de produire les samples un par un :
- chaque appel à next() on récupère un sample du FileReader
- ce sample est multiplié par la valeur du volume
- et le sample modifié est renvoyé

#### Play 

[**Source**](./src/play.rs)

Le fichier `Play` est responsable de la lecture audio via la carte son du système, en utilisant la bibliothèque `cpal`
Il est utilisé également comme un `wrapper` pour le volume, il contient un Volume encapsulé dans un `Arc<Mutex<Volume>>`.

Play est chargé de :
- sélectionner le périphérique audio
- créer un flux audio: output stream
- envoyer les samples audio à la carte son en temps réel  

Lors de l'appel a la fonction `play_samples()`, on initialise la carte son grâce à `cpal`, ensuite un stream audio est créé et une fonction (callback) fournie à CPAL est appelée en continu par le système audio et il remplit le buffer audio `(data: &mut [f32])` avec les samples fournis par Volume.

#### UICommandsReceiver 

[**Source**](./src/slint_logic.rs)

Utilisé pour recevoir les commandes et les exécuter. 

#### UICommandsSender

[**Source**](./src/slint_logic.rs)

Utilisé pour envoyer les commandes.
Définit une énumeration des différentes commandes.

### Enumération des commandes 

Chaque event listener sera trigger par une commande. 
Cela permettra d'utiliser le pattern matching sur une énumeration, ce qui permet une meilleure compréhension et une utilisation plus rapide (et idiomatique) de Rust.

Les différentes commandes sont les suivantes : 

- SwitchPlayMode() : Utilisée lors de l'appui sur le bouton "Play/Pause". 
- ChangeVolume(f32) : Utilisée lors d'un changement sur le slider du volume.
- ClickedInsidePopup(i32) : Utilisée lorsque l'utilisateur séléctionne un objet de la liste affichée par le bouton "Play file from project folder".
- LoadFile() : Utilisée lorsque l'utilisateur clique sur le bouton "Confirm" dans le popup. Il créé tous les objets Rust nécessaires au fonctionnent du flux.
- FetchDirectory() : Utilisée pour générer la liste des fichiers du dossier `data_samples`. 
- ImportFile() : Utilisée lorsque l'utilisateur clique sur le bouton "Import file from your home directory".
- UpdateProgressBar() : Utilisée par un timer défini dans Slint. Ce timer s'active quand la musique démarre, et s'arrête quand elle se finit. Permet de calculer l'avancée de la barre de progression.
- StopSamples() : Utilisée lorsque l'utilisateur clique sur le bouton "Stop". Cela détruit l'objet principal et en re-créée un avec les mêmes caractéristiques. 

