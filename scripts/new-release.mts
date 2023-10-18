#!/usr/bin/env ts-node --esm
import * as fs from "fs/promises";
import fetch from "node-fetch";
import path from "path";
import * as cp from "child_process";
import * as readline from "readline";

const dirname = new URL(".", import.meta.url).pathname;

const baseURL = new URL("https://api.github.com/repos/zwade/ruulang/");
const authToken = process.env.GITHUB_API_KEY!;

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
});

const readLine = () => {
    return new Promise<string>((resolve) => {
        rl.question("> ", (answer) => {
            resolve(answer);
        });
    });
}

const githubRequest = async (endpoint: string, payload?: unknown): Promise<any> => {
    const url = new URL(endpoint, baseURL);

    const response = await fetch(url.toString(), {
        headers: {
            authorization: `token ${authToken}`,
            accept: "application/vnd.github+json",
        },
        method: payload ? "POST" : "GET",
        body: payload ? JSON.stringify(payload) : undefined,
    });

    return response.json();
}

const execForResult = async (command: string): Promise<string> => {
    return new Promise((resolve, reject) => {
        cp.exec(command, (error, stdout, stderr) => {
            if (error) {
                reject(error);
            } else if (stderr) {
                reject(stderr);
            } else {
                resolve(stdout);
            }
        });
    });
}

const binaries = await fs.readdir(path.join(dirname, "../compiler/builds")).then((files) => files.filter((file) => !file.startsWith(".")));

const latest = await githubRequest("releases/latest");
const latestVersion = latest.tag_name;
console.log("Latest release:", latestVersion);

console.log("What should be the new version?");
const version = await readLine();

console.log("What should be the release title?");
const title = await readLine();

console.log("What should be the release body?");
const body = await readLine()

console.log(`
Creating a new tagged release!
    Version: ${version}
    Title: ${title}
    Body: ${body}
    Files:
        - ${binaries.join("\n        - ")}`);

console.log("Confirm? [y/N]");
const confirmation = await readLine();

if (confirmation !== "y") {
    console.log("Aborting...");
    process.exit(1);
}

await execForResult(`git tag ${version} && git push origin --tags ${version} || true`).catch((e) => {console.warn(e)});

const release = await githubRequest("releases", {
    tag_name: version,
    name: title,
    body,
    draft: true,
});

const uploadURL = new URL(release.upload_url.replace("{?name,label}", ""));

for (const file of binaries) {
    const filePath = path.join(dirname, "../compiler/builds", file);
    const fileBuffer = await fs.readFile(filePath);

    const fileUploadUrl = new URL(uploadURL);
    fileUploadUrl.searchParams.set("name", file);

    console.log(`Uploading ${file}...`);

    await fetch(fileUploadUrl.toString(), {
        headers: {
            authorization: `token ${authToken}`,
            accept: "application/vnd.github+json",
            "content-type": "application/octet-stream",
        },
        method: "POST",
        body: fileBuffer,
    });
}

process.exit(0);