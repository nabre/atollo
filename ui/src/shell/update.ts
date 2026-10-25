// What a check of Atollo's updates found, in words: Settings › Updates and
// the version in the status bar say the same.
import type { UpdateStatus } from "../ipc";
import { t } from "../i18n";

export function updateText(st: UpdateStatus): string {
  if (st.staged) return t("La versione {0} è pronta: sostituisce questa alla chiusura di Atollo.", st.staged);
  if (st.manual && st.latest)
    return t("La versione {0} è disponibile: scaricala dalla pagina della release.", st.latest);
  if (st.error) return t("Controllo non riuscito: {0}", st.error);
  if (st.latest && st.latest === st.current) return t("Atollo {0} è la versione più recente.", st.current);
  return t("Versione installata: {0}", st.current);
}

/** A newer version found: ready for the next start, or to download. */
export function updateFound(st: UpdateStatus): string | null {
  return st.staged ?? (st.manual ? st.latest : null);
}
