import os from "os";
import { commands, type ExtensionContext, window, workspace } from "vscode";
import {
    Executable,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from "vscode-languageclient/node";

import { resolveOrDownloadRuuLangServer } from "./serverControl.js";

let client: LanguageClient | undefined = undefined;

export const createLanguageServer = (command: string) => {
    const options: Executable = {
        command,
        transport: TransportKind.stdio,
        options: {
            env: { ...process.env, RUST_LOG: "debug" },
            cwd: workspace.workspaceFolders?.[0]?.uri?.fsPath ?? os.homedir(),
        },
    };

    const serverOptions: ServerOptions = {
        run: options,
        debug: options,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: "ruulang" }, { language: "plaintext" }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher("**/*.ruulang"),
        },
    };

    client = new LanguageClient("RuuLang", "RuuLang Language Server", serverOptions, clientOptions, true);
    return client;
};

export async function activate(context: ExtensionContext) {
    const restart = async (command: string | undefined) => {
        try {
            if (client) {
                await client.stop();
            }
        } catch (_e) {
            //pass
        }

        if (!command) {
            window.showErrorMessage("Failed to resolve RuuLang server");
            return;
        }

        client = createLanguageServer(command);
        client.start();
    };

    context.subscriptions.push(
        commands.registerCommand("extension.restart", async () => {
            const command = await resolveOrDownloadRuuLangServer(context, false);
            await restart(command);
            window.showInformationMessage("RuuLang Language Server restarted");
        }),
        commands.registerCommand("extension.redownload", async () => {
            const command = await resolveOrDownloadRuuLangServer(context, true);
            await restart(command);
            window.showInformationMessage("RuuLang Language Server redownloaded");
        }),
    );

    console.log("Starting server");
    const command = await resolveOrDownloadRuuLangServer(context);
    restart(command);
}

export function deactivate(): Thenable<void> | undefined {
    console.log("Deactivating server");
    if (!client) {
        return undefined;
    }
    return client.stop();
}
