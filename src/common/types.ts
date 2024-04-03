export interface ImportConfigurationPayload {
    configName: string;
    singleUse:  boolean;
    persistent: boolean;
    configFile: string;
}


export type Modals = null | "exit_confirmation";


export type MainAction = "exit_confirmation";
export type FromMainAction = {
    type: MainAction,
    data: unknown;
}