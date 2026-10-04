# Spécifications et Roadmap des Nouveaux Blocs pour `fsdr-cli`

Ce document définit les spécifications pour l'implémentation des prochains blocs FutureSDR, couvrant à la fois les fonctionnalités avancées de détection/déclenchement par étiquettes de flux (Stream Tags) et la roadmap de compatibilité `csdr` (Tiers 6 et 7).

---

# Partie I — Blocs Déclencheurs et Stream Tags (NFM / SigMF)

L'architecture repose sur l'utilisation d'**étiquettes de flux (Stream Tags)** qui se propagent de manière linéaire le long du flux d'échantillons pour permettre le déclenchement de commandes CLI basé sur la détection de signaux dans un flux Narrow FM (NFM).

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

Au moins un parmi `off_tag` ou `on_tag` doit être présent (les deux peuvent coexister).

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

## 5. Exemple d'Intégration en Ligne de Commande (Style `csdr`)

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

# Partie II — Roadmap de Remplacement CSDR (Tiers 6 & 7)

Cette section détaille les fonctionnalités prévues pour les prochains paliers d'implémentation de parité avec `csdr`.

---

## 🎯 **Tier 6 : Générateurs de Bruit, RTTY / UART & Modulation PSK**

Ce palier couvre trois grands domaines complémentaires :

### 1. **Tier 6A : Générateurs & Canaux de Bruit (Simulation & Tests RF)**
* **`gaussian_noise_c`** (`analog_gaussian_noise_c`) :
  * **Rôle :** Source génératrice de bruit blanc gaussien complexe (AWGN).
  * **Algorithme :** Générateur pseudo-aléatoire utilisant la transformée de Box-Muller ou la distribution normale standard $N(0, \sigma^2)$ sur $I$ et $Q$.
  * **Paramètres :** `variance` / `amplitude` (optionnel).
* **`uniform_noise_f`** (`analog_uniform_noise_f`) :
  * **Rôle :** Source génératrice de bruit uniforme sur $[-1.0, 1.0]$.
* **`awgn_cc`** (`analog_awgn_cc`) :
  * **Rôle :** Bloc de canal additif insérant du bruit blanc gaussien sur un signal IQ existant selon un niveau de bruit ou SNR spécifié ($y[k] = x[k] + n[k]$).
  * **Paramètres :** `snr` / `noise_variance`.

### 2. **Tier 6B : Décodage RTTY (Baudot) & Ligne Série (UART)**
* **`rtty_baudot2ascii_u8_u8`** (`digital_rtty_baudot2ascii_u8_u8`) :
  * **Rôle :** Décodeur du code ITA2 Baudot (5 bits) vers ASCII 8 bits.
  * **Logique :** Machine à états gérant les commutations de tables *Letters (LTRS, code 31)* et *Figures (FIGS, code 27)*, retour chariot et saut de ligne.
* **`generic_slicer_f_u8`** (`digital_generic_slicer_f_u8`) :
  * **Rôle :** Décisionnaire / trancheur de niveau de signal à seuils configurables ($y = \text{if } x > \text{threshold } 1 \text{ else } 0$).
  * **Paramètres :** `threshold` (float).
* **`serial_line_decoder_f_u8`** (`digital_serial_line_decoder_f_u8`) :
  * **Rôle :** Décodeur de flux série asynchrone (UART / NRZ) à partir d'échantillons suréchantillonnés.
  * **Paramètres :** `samples_per_baud`, `bits_per_word`, `stop_bits`.

### 3. **Tier 6C : Modulateurs Numériques PSK & Traitement Temporel**
* **`psk_modulator_u8_c`** (`digital_psk_modulator_u8_c`) :
  * **Rôle :** Modulateur PSK (BPSK, QPSK, M-PSK) associant chaque symbole ou octet binaire à un point de constellation complexe ($e^{j 2\pi m / M}$).
  * **Paramètres :** `constellation_size` (ex: 2 pour BPSK, 4 pour QPSK).
* **`psk31_interpolate_sine_cc`** (`digital_psk31_interpolate_sine_cc`) :
  * **Rôle :** Filtre de mise en forme sinusoïdale (Sine / Raised Cosine pulse shaping) spécifique au PSK31 pour lisser les transitions de phase et supprimer les lobes secondaires spectraux.
  * **Paramètres :** `samples_per_symbol`.
* **`duplicate_samples_ntimes_u8_u8`** (`blocks_duplicate_samples_ntimes_u8_u8`) :
  * **Rôle :** Suréchantillonneur temporel simple répétant chaque échantillon $N$ fois consécutives.
  * **Paramètres :** `n` (entier).

---

## 🚀 **Tier 7 : Filtrage Avancé, Interpolation & AGC Simple**

### 1. **Filtres de Mise en Forme & Détection de Crête**
* **`pulse_shaping_filter_cc` / `firdes_pulse_shaping_filter_f`** (`filter_pulse_shaping_cc`) :
  * **Rôle :** Filtre RRC (Root Raised Cosine) et calcul des coefficients optimisés pour transmission numérique sans interférence entre symboles (ISI).
  * **Paramètres :** `alpha` (roll-off, ex: 0.35), `samples_per_symbol`, `num_taps`.
* **`peaks_fir_cc` / `firdes_peak_c`** (`filter_peaks_fir_cc`) :
  * **Rôle :** Filtre FIR résonateur pour isolation ou élimination de raies spectrales étroites (détection de crêtes).

### 2. **Interpolateurs FIR & Taux d'Échantillonnage**
* **`plain_interpolate_cc` / `fir_interpolate_cc`** (`filter_fir_interpolate_cc`) :
  * **Rôle :** Interpolation entière de flux complexes avec filtrage polyphase ou filtre FIR passe-bas anti-repliement.
  * **Paramètres :** `interpolation_factor`.

### 3. **Contrôle Automatique de Gain (AGC) Spécialisé**
* **`simple_agc_cc`** (`analog_simple_agc_cc`) :
  * **Rôle :** AGC simplifié et ultra-rapide sur signal complexe IQ avec maintien de phase.
  * **Paramètres :** `rate`, `reference`.
* **`fastagc_ff`** (`analog_fastagc_ff`) :
  * **Rôle :** AGC à temps d'attaque et de rétablissement asymétriques et rapides pour signaux audio ou enveloppes.

---

## 📊 Récapitulatif de l'État des Paliers

| Palier | Thématique | Statut |
| :--- | :--- | :--- |
| **Tier 1** | Moteur FFT & Métriques spectrales (`fft_cc`, `fft_fc`, `logpower_cf`, `logaveragepower_cf`, `fft_exchange_sides_ff`) | ✅ **Complété & Validé** |
| **Tier 2** | Filtrage FIR & Translation spectrale (`dcblock_ff`, `decimating_shift_addition_cc`, `add_dcoffset_cc`, `firdes_*`) | ✅ **Complété & Validé** |
| **Tier 3** | Routage, Flux binaire & Écoulement (`repeat_u8`, `unpack_k_bits`, `flowcontrol`, `clone`, `through`, `none`) | ✅ **Complété & Validé** |
| **Tier 4** | Démodulation Numérique & Audio (`bpsk_costas_loop_cc`, `pll_cc`, `dbpsk_decoder`, `psk31_varicode_*`, `mono2stereo_s16`) | ✅ **Complété & Validé** |
| **Tier 5** | Modulation, ADPCM & Utilitaires (`encode/decode_ima_adpcm`, `compress_fft_adpcm`, `fmmod_fc`, `fixed_amplitude_cc`, `add_const_cc`, `differential_*`, `invert_u8`, `bfsk_demod_cf`, `detect_nan_ff`, `yes_f`) | ✅ **Complété & Validé** |
| **Tier 6** | Générateurs de bruit (AWGN), RTTY Baudot / UART & Modulateurs PSK | ⏳ **Prêt à démarrer** |
| **Tier 7** | Filtres RRC, Filtres de crête, Interpolateurs & AGC rapides | 📅 **Planifié** |
