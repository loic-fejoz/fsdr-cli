# Spécifications des Nouveaux Blocs pour `fsdr-cli`

Ce document définit les spécifications pour l'implémentation de quatre nouveaux blocs FutureSDR destinés à permettre le déclenchement de commandes CLI basé sur la détection de signaux dans un flux Narrow FM (NFM).

L'architecture repose sur l'utilisation d'**étiquettes de flux (Stream Tags)** qui se propagent de manière linéaire le long du flux d'échantillons.

---

## 1. Détecteur CTCSS (`ctcss_detect_ff`)

* **ID GRC :** `analog_ctcss_detect_ff`
* **Rôle :** Analyser le flux audio démodulé et marquer le début de la détection d'une tonalité sub-audible CTCSS par un tag de début.

### Paramètres
* `samp_rate` (expression / float) : Taux d'échantillonnage de l'entrée.
* `tone_freq` (float) : Fréquence de la tonalité CTCSS cible en Hz (ex: `88.5`).
* `threshold` (float) : Seuil de puissance de la tonalité filtrée pour valider la détection.
* `duration` (expression / duration) : Temps de validation continu (ex: `150ms`, traduit en échantillons) pour éviter les faux déclenchements.
* `tag_name` (string) : Nom du tag à insérer sur détection (par défaut : `"msgstart"`).

### Entrée / Sortie
* **Entrée :** Flux `f32` (Audio démodulé).
* **Sortie :** Flux `f32` (Identique à l'entrée, avec ajout de l'étiquette `tag_name` à l'index exact du début du message).

### Logique Interne
1. Appliquer un filtre passe-bande étroit (type IIR ou Goertzel) centré sur `tone_freq`.
2. Calculer la puissance moyenne de la tonalité filtrée.
3. Si la puissance dépasse `threshold` en continu durant `duration` (exprimé en nombre d'échantillons : $D = \text{duration} \times \text{samp\_rate}$) :
   * Lever l'état "Détecté".
   * Si l'état précédent était "Non détecté" (front montant), insérer l'étiquette `tag_name` sur l'échantillon courant.

---

## 2. Détecteur de Squelch / Puissance (`power_tagger_cc`)

* **ID GRC :** `analog_power_tagger_cc`
* **Rôle :** Surveiller la puissance moyenne du signal et insérer un tag de fin lorsque le signal apparait/disparaît (perte de porteuse).

At least one of `off_tag` or `on_tag` must be present. Both can co-exist.

### Paramètres
* `samp_rate` (expression / float) : Taux d'échantillonnage de l'entrée.
* `threshold` (expression / float) : Configure à la fois `off_threshold` et `on_threshold` à la même valeur.
* `delay` (expression / duration) : Configure à la fois `off_delay` et `on_delay` à la même valeur.
* `off_threshold` (expression / float) : Seuil de puissance en dB ou valeur brute en dessous duquel le squelch se ferme.
* `off_delay` (expression / duration) : Durée de maintien (ex: `300ms`, traduit en échantillons) pendant laquelle la puissance doit rester sous le seuil avant de valider la fermeture du squelch.
* `on_threshold` (expression / float) : Seuil de puissance en dB ou valeur brute au dessus duquel le squelch s'ouvre.
* `on_delay` (expression / duration) : Durée de maintien (ex: `300ms`, traduit en échantillons) pendant laquelle la puissance doit rester au-dessus du seuil avant de valider l'ouverture du squelch.
* `window_size` (expression / duration) : Fenêtre d'intégration pour le calcul de la puissance moyenne (ex: `10ms`).
* `off_tag` (string) : Nom du tag à insérer sur perte de puissance (par défaut : `"msgend"`).
* `on_tag` (string) : Nom du tag à insérer sur découverte de puissance (par défaut : `"msgstart"`).

### Entrée / Sortie
* **Entrée :** Flux `Complex32` (IQ RF/IF) ou `f32` (Audio).
* **Sortie :** Flux identique (les échantillons sont transmis inchangés, avec ajout de l'étiquette `tag_name` sur le front descendant).

### Logique Interne
1. Calculer la puissance moyenne du signal sur une fenêtre glissante de `window_size`.
2. Si la puissance passe en dessous de `off_threshold` :
   * Démarrer un compteur d'échantillons correspondant à `off_delay`.
   * Si la puissance remonte au-dessus du seuil avant la fin du compteur, réinitialiser.
   * Si la puissance reste sous le seuil en continu pendant `off_delay` (front descendant validé) :
     * Émettre l'étiquette `tag_name` à l'index de début de la perte de puissance.

---

## 3. Temporisateur de Sécurité (`timer_tagger_ff`)

* **ID GRC :** `blocks_timer_tagger_ff`
* **Rôle :** Insérer automatiquement un tag de fin après une durée maximale de transmission si aucun tag de fin naturel (comme la perte de puissance) n'a été observé.

### Paramètres
* `samp_rate` (expression / float) : Taux d'échantillonnage du flux.
* `start_tag` (string) : Nom du tag qui démarre le minuteur (ex: `"msgstart"`).
* `end_tag` (string) : Nom du tag de fin attendu ou à générer (ex: `"msgend"`).
* `duration` (expression / duration) : Limite de temps max (ex: `30s`, traduit en échantillons).

### Entrée / Sortie
* **Entrée :** Flux générique `T` (avec tags).
* **Sortie :** Flux générique `T` (identique, avec ajout éventuel du tag `end_tag`).

### Logique Interne
1. À la réception de `start_tag` à l'index $I_{start}$ :
   * Programmer une échéance d'échantillons à l'index $I_{timeout} = I_{start} + (\text{duration} \times \text{samp\_rate})$.
2. Si un tag `end_tag` est détecté dans le flux d'entrée à un index $I < I_{timeout}$ :
   * Annuler la planification du timeout en cours (le tag de fin naturel est prioritaire).
3. Si l'index courant atteint $I_{timeout}$ sans avoir vu passer de `end_tag` :
   * Insérer un tag `end_tag` à l'index $I_{timeout}$.

---

## 4. Déclencheur CLI / Enregistreur SigMF (`cmd_trigger_f`)

* **ID GRC :** `blocks_cmd_trigger_f`
* **Rôle :** Agir comme un Sink intelligent qui écrit la portion utile du signal dans des fichiers temporaires au format SigMF, puis lance de façon asynchrone une commande système externe.

### Paramètres
* `start_tag` (string) : Tag de début d'enregistrement.
* `end_tag` (string) : Tag de fin d'enregistrement et déclencheur CLI.
* `cmd` (string) : Ligne de commande à exécuter (ex: `'./mon-script.sh $input_file'`). Le placeholder `$input_file` sera remplacé par le chemin absolu du fichier `.sigmf-meta` (ou `.sigmf-data` selon besoin).
* `datatype` (DatasetFormat) : Type de données SigMF (ex: `cf32_le` ou `f32_le`).
* `samp_rate` (expression / float) : Taux d'échantillonnage à inscrire dans les métadonnées SigMF.

### Entrée / Sortie
* **Entrée :** Flux générique `T` (avec tags).
* **Sortie :** Aucune (Sink).

### Logique Interne
* **État Inactif (Attente) :**
  * Si un tag `end_tag` ou des échantillons normaux passent, ils sont ignorés.
  * Si le tag `start_tag` est détecté à l'index $N$ :
    * Créer un répertoire temporaire unique via `tempfile::tempdir()`.
    * Initialiser un enregistreur SigMF (réutilisation du code de `SigMFSink` pour générer `message.sigmf-data` et `message.sigmf-meta`).
    * Passer à l'état **Actif (Enregistrement)**.
    * Commencer à écrire les échantillons dans le fichier à partir de l'échantillon portant le tag `start_tag`.

* **État Actif (Enregistrement) :**
  * Écrire tous les échantillons entrants dans le fichier SigMF temporaire.
  * Si un nouveau tag `start_tag` est reçu : l'ignorer.
  * Si le tag `end_tag` est détecté :
    * Arrêter l'écriture et finaliser les fichiers SigMF (`.sigmf-data` et `.sigmf-meta`).
    * Préparer la commande CLI en substituant `$input_file` par le chemin absolu du fichier généré.
    * Lancer le processus de manière asynchrone :
      ```rust
      let child = std::process::Command::new("sh")
          .arg("-c")
          .arg(&resolved_cmd)
          .spawn()?;
      ```
    * Lancer un thread d'arrière-plan dédié pour surveiller la fin de la commande sans bloquer le traitement SDR, afin d'assurer le nettoyage automatique du répertoire temporaire à la fin de l'exécution :
      ```rust
      let temp_dir = self.current_temp_dir.take();
      std::thread::spawn(move || {
          let _ = child.wait();
          // Le drop de temp_dir à la fin de ce scope supprime les fichiers temporaires
      });
      ```
    * Repasser à l'état **Inactif (Attente)**.

---

---

## 5. Exemple d'Intégration en Ligne de Commande (Style `csdr`)

Voici la commande finale et fonctionnelle utilisant la syntaxe réelle des blocs existants de `fsdr-cli` combinée aux nouveaux blocs :

```bash
export SAMPLING=2400000
rtl_sdr -f 144500000 -s ${SAMPLING} -g 35 - | \
fsdr-cli csdr convert_u8_f ! convert_ff_c \
  ! rational_resampler_cc 1 50 \
  ! power_tagger_cc --samp-rate "${SAMPLING}/50" --off-threshold -20dB --off-delay "300ms" --off-tag msgend \
  ! fmdemod_quadri_cf \
  ! deemphasis_nfm_ff 48000 \
  ! ctcss_detect_ff --samp-rate "${SAMPLING}/50" --tone 88.5 --threshold 0.000002 --duration "150ms" --tag msgstart \
  ! timer_tagger_ff --samp-rate "${SAMPLING}/50" --duration "30s" --start msgstart --end msgend \
  ! cmd_trigger_f --start-tag msgstart --end-tag msgend --cmd './mon-script.sh $input_file'
```

---

## 6. Remarques de Cohérence et Points de Vigilance Technique

1. **Pré-requis de propagation des tags (Essentiel) :**
   * Tous les blocs intermédiaires de la chaîne (comme `rational_resampler_cc`, `fmdemod_quadri_cf`, `deemphasis_nfm_ff`) **doivent impérativement propager et transférer correctement les tags**.
   * Les blocs effectuant des changements de taux d'échantillonnage (décimation ou interpolation) doivent mettre à l'échelle dynamiquement l'index de chaque tag ($I_{out} = I_{in} \times \frac{\text{interp}}{\text{decim}}$) pour éviter toute désynchronisation.

2. **Évaluation des expressions (Pest Grammar) :**
   * La syntaxe proposée utilise des variables d'environnement (`"${SAMPLING}"`) et des expressions mathématiques (`"${SAMPLING}/50"`). L'évaluateur [`Grc2FutureSdr::parameter_as_f64`](file:///home/loic/projets/fsdr-cli/src/grc/converter/mod.rs) doit être capable d'interpréter ces divisions mathématiques après substitution de la variable par l'interpréteur de commandes (Shell).
   * La grammaire du fichier [`src/cmd_line.pest`](file:///home/loic/projets/fsdr-cli/src/cmd_line.pest) n'est pas modifiée pour les blocs existants (comme `rational_resampler_cc 1 50` ou `deemphasis_nfm_ff 48000`) afin de conserver la stabilité du parser de commandes historique.
