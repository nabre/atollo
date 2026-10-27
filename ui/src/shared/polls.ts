// The commands that more than one part of the window reads every few
// seconds, each one poll for all of them (./poll): the Activity counts of
// the app and the pages of the database servers, the web servers, the local
// models and the agents; the VM's state for the status bar and the Web
// Servers page.
import { agentsStatus, isolationStatus, llmStatus, serversList, servicesList } from "../ipc";
import { createPoll } from "./poll";

/** `services_list`: the database servers and their versions. */
export const servicesPoll = createPoll(() => servicesList());

/** `servers_list` without a new scan: the web server pools. */
export const serversPoll = createPoll(() => serversList());

/** `llm_status`: the server of the local models. */
export const llmPoll = createPoll(() => llmStatus());

/** `agents_status`: the coding agents and their sign-in. */
export const agentsPoll = createPoll(() => agentsStatus());

/** `isolation_status` without forcing `sbx`: the VM's state. */
export const isolationPoll = createPoll(() => isolationStatus());
