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

let client: LanguageClient;

export async function activate(context: ExtensionContext) {
    console.log("Activating server");

    const command = await resolveOrDownloadRuuLangServer(context);

    if (!command) {
        window.showErrorMessage("Failed to resolve RuuLang server");
        return;
    }

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

    client.start();

    context.subscriptions.push(
        commands.registerCommand("extension.restart", async () => {
            await client.stop();
            await client.start();
            window.showInformationMessage("RuuLang Language Server restarted");
        }),
    );
}

export function deactivate(): Thenable<void> | undefined {
    console.log("Deactivating server");
    if (!client) {
        return undefined;
    }
    return client.stop();
}
