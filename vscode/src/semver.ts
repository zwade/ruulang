export class SemVer {
    private major;
    private minor;
    private patch;

    constructor(major: number, minor: number, patch: number) {
        this.major = major;
        this.minor = minor;
        this.patch = patch;
    }

    public static parse(version: string) {
        const match = version.match(/^v(\d+)\.(\d+)\.(\d+)$/);
        if (!match) {
            return null;
        }

        const [, major, minor, patch] = match;
        return new SemVer(+major, +minor, +patch);
    }

    public static get zero() {
        return new SemVer(0, 0, 0);
    }

    public static compare(a: SemVer, b: SemVer) {
        if (a.major !== b.major) {
            return a.major - b.major;
        }

        if (a.minor !== b.minor) {
            return a.minor - b.minor;
        }

        return a.patch - b.patch;
    }

    public static equals(a: SemVer, b: SemVer) {
        return a.major === b.major && a.minor === b.minor && a.patch === b.patch;
    }
}
