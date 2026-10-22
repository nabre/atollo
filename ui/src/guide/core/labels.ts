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
  };
}

export type GuideLabels = ReturnType<typeof guideLabels>;
