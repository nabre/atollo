// The database engines' names, as the pages show them.
export const ENGINE_NAMES: Record<"mysql" | "mongodb", string> = { mysql: "MySQL", mongodb: "MongoDB" };

/** An engine's name, or its id for an engine the interface does not know. */
export function engineName(id: string): string {
  return (ENGINE_NAMES as Record<string, string | undefined>)[id] ?? id;
}
