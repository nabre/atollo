// The words the Guide's widgets write by themselves («In breve», «Consiglio»,
// «Argomenti collegati», «Copia»…), one copy for every renderer. A function,
// called while drawing: the texts follow the current language.
import { tr } from "./lang";

export function guideLabels() {
  return {
    inShort: tr("In breve", "In short", "En bref", "Kurz gesagt"),
    tip: tr("Consiglio", "Tip", "Conseil", "Tipp"),
    info: tr("Da sapere", "Good to know", "À savoir", "Gut zu wissen"),
    warn: tr("Attenzione", "Watch out", "Attention", "Achtung"),
    under: tr("Sotto il cofano", "Under the hood", "Sous le capot", "Unter der Haube"),
    related: tr("Argomenti collegati", "Related topics", "Sujets liés", "Verwandte Themen"),
    example: tr(
      "Esempio pratico nello screenshot",
      "Practical example in screenshot",
      "Exemple pratique dans la capture d'écran",
      "Praktisches Beispiel im Screenshot",
    ),
    sample: tr("dati di esempio", "sample data", "données d'exemple", "Beispieldaten"),
    zoom: tr("Ingrandisci", "Zoom", "Agrandir", "Vergrößern"),
    zoomHint: tr(
      "Clicca per ingrandire l'immagine",
      "Click to enlarge image",
      "Cliquer pour agrandir l'image",
      "Klicken, um das Bild zu vergrößern",
    ),
    close: tr("Chiudi", "Close", "Fermer", "Schließen"),
    copy: tr("Copia", "Copy", "Copier", "Kopieren"),
    copied: tr("Copiata", "Copied", "Copiée", "Kopiert"),
    learnMore: tr("Approfondisci:", "Learn more:", "En savoir plus :", "Mehr dazu:"),
    read: tr("Leggi", "Read", "Lire", "Lesen"),
    kit: {
      title: tr("Il tuo kit", "Your kit", "Votre kit", "Dein Kit"),
      version: tr("Versione", "Version", "Version", "Version"),
      root: tr("Cartella del kit", "Kit folder", "Dossier du kit", "Ordner des Kits"),
      projects: tr("Cartella dei progetti", "Projects folder", "Dossier des projets", "Projektordner"),
      diskFs: tr(
        "File system del disco",
        "Disk file system",
        "Système de fichiers du disque",
        "Dateisystem der Festplatte",
      ),
    },
    sources: {
      columns: [
        tr("Componente", "Component", "Composant", "Komponente"),
        tr("Versioni da", "Versions from", "Versions depuis", "Versionen von"),
        tr("File da", "Files from", "Fichiers depuis", "Dateien von"),
        tr("Verifica", "Check", "Vérification", "Prüfung"),
        tr(
          "In data\\apps\\<sistema>\\",
          "In data\\apps\\<system>\\",
          "Dans data\\apps\\<système>\\",
          "In data\\apps\\<System>\\",
        ),
      ],
      all: tr("Tutti", "All", "Tous", "Alle"),
      filter: tr(
        "Filtra per nome, indirizzo, impronta…",
        "Filter by name, address, hash…",
        "Filtrer par nom, adresse, empreinte…",
        "Nach Name, Adresse, Hash filtern…",
      ),
      website: tr("Sito", "Website", "Site", "Website"),
      installed: tr("installato", "installed", "installé", "installiert"),
      notHere: tr("non disponibile qui", "not available here", "non disponible ici", "hier nicht verfügbar"),
      notNeeded: tr("non serve qui", "not needed here", "pas nécessaire ici", "hier nicht nötig"),
      notInstalled: tr("non installato", "not installed", "non installé", "nicht installiert"),
      withComponent: tr("con il suo componente", "with its component", "avec son composant", "mit seiner Komponente"),
    },
  };
}

export type GuideLabels = ReturnType<typeof guideLabels>;
