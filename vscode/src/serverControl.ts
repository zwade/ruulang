import * as fs from "fs/promises";
import { ExtensionContext, ProgressLocation, window } from "vscode";

import { SemVer } from "./semver.js";

interface LatestRelease {
    tag_name: string;
    assets: {
        url: string;
        name: string;
    }[];
}

type Arch = "aarch64" | "x86_64" | "unknown";
type Platform = "unknown-linux-gnu" | "apple-darwin" | "unknown";

const getArch = (): Arch => {
    switch (process.arch) {
        case "arm64":
            return "aarch64";
        case "x64":
            return "x86_64";
        default:
            return "unknown";
    }
};

const getPlatform = (): Platform => {
    switch (process.platform) {
        case "darwin":
            return "apple-darwin";
        case "linux":
            return "unknown-linux-gnu";
        default:
            return "unknown";
    }
};

const getTriple = (): string => {
    return `${getArch()}-${getPlatform()}`;
};

export const downloadLatestRuuLangServer = async (context: ExtensionContext, currentVersion: SemVer) => {
    const { default: fetch } = await import("node-fetch");

    const baseUri = new URL("https://api.github.com/repos/zwade/ruulang/");
    const latestReleaseUri = new URL("releases/latest", baseUri);

    const releaseResponse = await fetch(latestReleaseUri.toString(), {
        headers: { accept: "application/vnd.github+json" },
    });
    const release = (await releaseResponse.json()) as LatestRelease;
    console.log(release, latestReleaseUri.toString());
    const releaseVersion = SemVer.parse(release.tag_name);

    if (!releaseVersion || SemVer.compare(releaseVersion, currentVersion) <= 0) {
        return undefined;
    }

    const neededFile = `ruulang-server.${getTriple()}`;

    const asset = release.assets.find((x) => x.name === neededFile);
    if (!asset) {
        throw new Error(`Could not find latest ruulang-server release for platform: ${getTriple()}`);
    }

    const assetUri = new URL(asset.url);
    const assetResponse = await fetch(assetUri.toString(), {
        headers: { accept: "application/octet-stream" },
    });

    const binaryRoot = context.asAbsolutePath("./bin");
    await fs.mkdir(binaryRoot, { recursive: true });

    const binaryPath = context.asAbsolutePath(`./bin/${neededFile}.${release.tag_name}`);
    const binary = await assetResponse.arrayBuffer();

    await fs.writeFile(binaryPath, Buffer.from(binary));
    await fs.chmod(binaryPath, 0o755);

    return binaryPath;
};

export const resolveOrDownloadRuuLangServer = async (context: ExtensionContext) => {
    if (process.env.DEBUG_MODE === "true") {
        return context.asAbsolutePath("../compiler/target/debug/ruulang-server");
    }

    const binaryRoot = context.asAbsolutePath("./bin");
    await fs.mkdir(binaryRoot, { recursive: true });

    const files = await fs.readdir(binaryRoot);
    const possibleServers = files
        .map((file): [SemVer, string] | null => {
            const match = file.match(/^ruulang-server\.v(\d+)\.(\d+)\.(\d+)$/);
            if (!match) {
                return null;
            }

            const [, major, minor, patch] = match;
            return [new SemVer(+major, +minor, +patch), file];
        })
        .filter((x): x is NonNullable<typeof x> => x !== null);

    const latestServer = possibleServers.sort(([a], [b]) => SemVer.compare(a, b)).pop();
    if (latestServer) {
        // Run this in the background
        downloadLatestRuuLangServer(context, latestServer[0]);

        return context.asAbsolutePath(`./bin/${latestServer[1]}`);
    }

    const newFile = await window.withProgress(
        { cancellable: false, location: ProgressLocation.Notification, title: "Downloading latest ruulang server" },
        () => downloadLatestRuuLangServer(context, SemVer.zero),
    );

    return newFile;
};
