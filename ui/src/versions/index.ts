// Versions of the languages kept side by side (DECISIONS.md D-80): the
// default folder, replaced by updates, and one saved folder per minor line
// (an update to a new minor keeps the old one; others can be downloaded).
// The Tools page chooses the kit's line of each language in its row, the
// project page the project's. The lists propose also the lines the source offers and
// are not on disk yet: choosing one downloads it.
export { composerOf, splitVersion } from "./model";
export { ToolVersions, useKitVersions, type KitVersions } from "./kit";
export { ProjectVersions } from "./project";
