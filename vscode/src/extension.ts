import { type ExtensionContext, commands, workspace, window } from "vscode";
import os from "os";

import {
    Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    console.log("Activating server")
    const command =
        process.env.DEBUG_MODE === "true"
            ? context.asAbsolutePath("./bin/ruulang-server-debug")
            : context.asAbsolutePath("./bin/ruulang-server");

    const options: Executable = {
        command,
        transport: TransportKind.stdio,
        options: {
            env: { ...process.env, RUST_LOG: "debug" },
            cwd: workspace.workspaceFolders?.[0]?.uri?.fsPath ?? os.homedir(),
        },
    }

    const serverOptions: ServerOptions = {
        run: options,
        debug: options,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: "ruulang" }, { language: "plaintext" }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher("**/*.ruulang"),
        }
    };

    context.subscriptions.push(commands.registerCommand("extension.sayHello", () => window.showInformationMessage('Hello World!')));

    client = new LanguageClient(
        "RuuLang",
        "RuuLang Language Server",
        serverOptions,
        clientOptions,
        true
    );

    client.start();

    context.subscriptions.push(commands.registerCommand("extension.restart", async () => {
        await client.stop();
        await client.start();
        window.showInformationMessage("RuuLang Language Server restarted");
    }));
}

export function deactivate(): Thenable<void> | undefined {
    console.log("Deactivating server");
    if (!client) {
        return undefined;
    }
    return client.stop();
}
